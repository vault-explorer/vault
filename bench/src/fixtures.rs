//! Deterministic test trees: the same [`TreeSpec`] and seed produce the same
//! paths, sizes, timestamps and contents on every OS.
//!
//! [`plan`] decides what a tree contains without touching the disk, [`generate`]
//! writes it and [`verify`] checks a tree on disk against its manifest. Trees
//! live under [`default_root`], which is gitignored.
//!
//! Changing anything that alters the generated output means bumping
//! [`GENERATOR_VERSION`] and updating the golden digests in the tests.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Bumped whenever the generator's output changes for an existing spec and seed.
pub const GENERATOR_VERSION: u32 = 1;

/// Seed used by the benches and by default on the command line.
pub const DEFAULT_SEED: u64 = 0x5641_554C_5400_0001;

/// 10,000 files in one folder.
pub const FLAT_10K: TreeSpec = TreeSpec {
    name: "flat-10k",
    files: 10_000,
    shape: Shape::Flat,
};

/// 100,000 files in one folder.
pub const FLAT_100K: TreeSpec = TreeSpec {
    name: "flat-100k",
    files: 100_000,
    shape: Shape::Flat,
};

/// 1,000,000 files in a nested tree. Opt-in only: slow to write and large on
/// disk, so it is never part of [`DEFAULT_SPECS`] or CI.
pub const NESTED_1M: TreeSpec = TreeSpec {
    name: "nested-1m",
    files: 1_000_000,
    shape: Shape::Nested {
        dirs: 50_000,
        max_depth: 10,
    },
};

/// Trees generated unless the 1M tree is explicitly requested.
pub const DEFAULT_SPECS: [TreeSpec; 2] = [FLAT_10K, FLAT_100K];

/// Every known tree.
pub const ALL_SPECS: [TreeSpec; 3] = [FLAT_10K, FLAT_100K, NESTED_1M];

/// Unix time of 2021-01-01; file timestamps are spread over the five years after it.
const MODIFIED_EPOCH_SECS: u64 = 1_609_459_200;
const MODIFIED_SPREAD_SECS: u64 = 5 * 365 * 24 * 3600;

/// What a tree looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSpec {
    /// Directory name of the tree under the fixtures root.
    pub name: &'static str,
    /// Number of files.
    pub files: usize,
    /// How the files are arranged.
    pub shape: Shape,
}

/// How a tree's files are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Every file directly in the tree's root.
    Flat,
    /// Files spread unevenly over a directory tree.
    Nested {
        /// Number of directories, excluding the root.
        dirs: usize,
        /// Deepest directory level below the root.
        max_depth: usize,
    },
}

/// One file in a [`Plan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    /// Path relative to the tree's root, separated by `/`.
    pub rel_path: String,
    /// Size in bytes.
    pub size: u64,
    /// Last-modified time, in seconds since the Unix epoch.
    pub modified_secs: u64,
    content_seed: u64,
}

/// Everything a tree contains, in creation order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Directories relative to the tree's root; parents come before children.
    pub dirs: Vec<String>,
    /// All files.
    pub files: Vec<PlannedFile>,
}

impl Plan {
    /// Sum of all file sizes.
    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    fn digest(&self) -> String {
        tree_digest(
            self.files
                .iter()
                .map(|f| (f.rel_path.clone(), f.size, f.modified_secs))
                .collect(),
        )
    }
}

/// Records a completed tree. Stored beside the tree, not inside it, so it never
/// shows up in a listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// [`TreeSpec::name`].
    pub name: String,
    /// [`GENERATOR_VERSION`] that wrote the tree.
    pub generator_version: u32,
    /// Seed the tree was generated from.
    pub seed: u64,
    /// Number of files.
    pub files: usize,
    /// Number of directories, excluding the root.
    pub dirs: usize,
    /// Sum of all file sizes.
    pub bytes: u64,
    /// Digest of every path, size and timestamp.
    pub digest: String,
    /// File count per extension.
    pub extensions: BTreeMap<String, usize>,
}

/// Errors from generating or checking fixtures.
#[derive(Debug)]
#[non_exhaustive]
pub enum FixtureError {
    /// A file-system operation failed.
    Io {
        /// The path being accessed.
        path: PathBuf,
        /// The underlying OS error.
        source: io::Error,
    },
    /// A manifest could not be read or written as JSON.
    Manifest {
        /// The manifest's path.
        path: PathBuf,
        /// The underlying error.
        source: serde_json::Error,
    },
    /// The tree hasn't been generated, or was generated with other parameters.
    NotGenerated {
        /// The tree that was requested.
        name: String,
        /// What is wrong.
        reason: String,
    },
    /// A directory is in the way that this generator didn't create.
    Foreign {
        /// The directory, which is left untouched.
        path: PathBuf,
    },
    /// The tree on disk doesn't match its manifest.
    Mismatch {
        /// The tree that was checked.
        name: String,
        /// What differs.
        reason: String,
    },
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Manifest { path, source } => write!(f, "{}: {source}", path.display()),
            Self::NotGenerated { name, reason } => write!(
                f,
                "fixture {name} is not usable ({reason}). Generate it with: \
                 cargo run -p vault-bench --release --bin vault-fixtures -- generate"
            ),
            Self::Foreign { path } => write!(
                f,
                "{} exists but was not created by this generator; move it or pick another root",
                path.display()
            ),
            Self::Mismatch { name, reason } => write!(f, "fixture {name} is corrupt: {reason}"),
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Manifest { source, .. } => Some(source),
            Self::NotGenerated { .. } | Self::Foreign { .. } | Self::Mismatch { .. } => None,
        }
    }
}

/// `$VAULT_BENCH_FIXTURES`, or `bench/fixtures` (gitignored).
#[must_use]
pub fn default_root() -> PathBuf {
    std::env::var_os("VAULT_BENCH_FIXTURES").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures"),
        PathBuf::from,
    )
}

/// Decides what a tree contains. Pure: no disk access.
#[must_use]
pub fn plan(spec: &TreeSpec, seed: u64) -> Plan {
    let mut rng = Rng::new(seed ^ fnv1a(FNV_OFFSET, spec.name.as_bytes()));
    let tree = match spec.shape {
        Shape::Flat => DirTree::flat(),
        Shape::Nested { dirs, max_depth } => DirTree::nested(&mut rng, dirs, max_depth),
    };

    let mut taken = HashSet::with_capacity(spec.files);
    let files = (0..spec.files)
        .map(|index| {
            let dir = tree.pick_for_file(&mut rng);
            let generated = generate_name(&mut rng);
            let name = unique_name(generated.name, index, dir, &mut taken);
            let rel_path = match tree.paths[dir].as_str() {
                "" => name,
                parent => format!("{parent}/{name}"),
            };
            PlannedFile {
                rel_path,
                size: generated.size.sample(&mut rng),
                modified_secs: MODIFIED_EPOCH_SECS + rng.below(MODIFIED_SPREAD_SECS),
                content_seed: rng.next_u64(),
            }
        })
        .collect();

    Plan {
        dirs: tree.paths.into_iter().skip(1).collect(),
        files,
    }
}

/// Returns `count` realistic, unique file names without touching the disk.
#[must_use]
pub fn name_corpus(count: usize, seed: u64) -> Vec<String> {
    let spec = TreeSpec {
        name: "name-corpus",
        files: count,
        shape: Shape::Flat,
    };
    plan(&spec, seed)
        .files
        .into_iter()
        .map(|f| f.rel_path)
        .collect()
}

/// Writes a tree under `root` and returns its manifest.
///
/// A tree that already matches is left alone unless `force` is set. A tree left
/// behind by an interrupted run is replaced. A directory this generator didn't
/// create is never removed.
///
/// # Errors
///
/// Returns [`FixtureError::Foreign`] if something else occupies the tree's path,
/// or [`FixtureError::Io`] if writing fails.
pub fn generate(
    root: &Path,
    spec: &TreeSpec,
    seed: u64,
    force: bool,
) -> Result<Manifest, FixtureError> {
    if !force && let Ok(manifest) = require(root, spec, seed) {
        return Ok(manifest);
    }
    remove(root, spec)?;

    let tree_dir = root.join(spec.name);
    create_dir(&tree_dir)?;
    let partial = partial_marker_path(root, spec);
    fs::write(&partial, b"").map_err(io_err(&partial))?;

    let plan = plan(spec, seed);
    for dir in &plan.dirs {
        create_dir(&tree_dir.join(dir))?;
    }
    let mut buffer = Vec::new();
    for file in &plan.files {
        write_file(&tree_dir, file, &mut buffer)?;
    }

    let manifest = manifest_for(spec, seed, &plan);
    let path = manifest_path(root, spec);
    let json = serde_json::to_vec_pretty(&manifest).map_err(|source| FixtureError::Manifest {
        path: path.clone(),
        source,
    })?;
    fs::write(&path, json).map_err(io_err(&path))?;
    fs::remove_file(&partial).map_err(io_err(&partial))?;
    Ok(manifest)
}

/// Checks that a tree was generated with `spec` and `seed`, without reading the
/// tree itself. Returns its manifest and is cheap enough to call before a bench.
///
/// # Errors
///
/// Returns [`FixtureError::NotGenerated`] if the tree is missing, incomplete or
/// from another generator version or seed.
pub fn require(root: &Path, spec: &TreeSpec, seed: u64) -> Result<Manifest, FixtureError> {
    let not_generated = |reason: String| FixtureError::NotGenerated {
        name: spec.name.to_owned(),
        reason,
    };
    let path = manifest_path(root, spec);
    if partial_marker_path(root, spec).exists() {
        return Err(not_generated("a previous run was interrupted".to_owned()));
    }
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(not_generated(format!("no manifest at {}", path.display())));
        }
        Err(source) => return Err(FixtureError::Io { path, source }),
    };
    let manifest: Manifest =
        serde_json::from_str(&text).map_err(|source| FixtureError::Manifest { path, source })?;
    if manifest.generator_version != GENERATOR_VERSION {
        return Err(not_generated(format!(
            "generated by version {}, this is version {GENERATOR_VERSION}",
            manifest.generator_version
        )));
    }
    if manifest.seed != seed {
        return Err(not_generated(format!(
            "generated with seed {}, expected {seed}",
            manifest.seed
        )));
    }
    Ok(manifest)
}

/// Walks a tree on disk and checks every path, size and timestamp against its
/// manifest.
///
/// # Errors
///
/// Returns [`FixtureError::Mismatch`] if anything differs, and the errors of
/// [`require`] if the tree isn't generated.
pub fn verify(root: &Path, spec: &TreeSpec, seed: u64) -> Result<Manifest, FixtureError> {
    let manifest = require(root, spec, seed)?;
    let mut found = Vec::with_capacity(manifest.files);
    let mut dirs = 0;
    let mut pending = vec![(root.join(spec.name), String::new())];
    while let Some((dir, prefix)) = pending.pop() {
        for entry in fs::read_dir(&dir).map_err(io_err(&dir))? {
            let entry = entry.map_err(io_err(&dir))?;
            let path = entry.path();
            let rel_path = format!("{prefix}{}", entry.file_name().to_string_lossy());
            let metadata = entry.metadata().map_err(io_err(&path))?;
            if metadata.is_dir() {
                dirs += 1;
                pending.push((path, format!("{rel_path}/")));
            } else {
                let modified = metadata.modified().map_err(io_err(&path))?;
                let secs = modified
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                found.push((rel_path, metadata.len(), secs));
            }
        }
    }

    let mismatch = |reason: String| FixtureError::Mismatch {
        name: spec.name.to_owned(),
        reason,
    };
    if found.len() != manifest.files || dirs != manifest.dirs {
        return Err(mismatch(format!(
            "found {} files and {dirs} dirs, expected {} and {}",
            found.len(),
            manifest.files,
            manifest.dirs
        )));
    }
    let digest = tree_digest(found);
    if digest != manifest.digest {
        return Err(mismatch(format!(
            "digest {digest}, expected {}",
            manifest.digest
        )));
    }
    Ok(manifest)
}

/// Deletes a tree and its manifest. Does nothing if the tree doesn't exist.
///
/// # Errors
///
/// Returns [`FixtureError::Foreign`] if the directory has neither a manifest nor
/// an interrupted-run marker, and [`FixtureError::Io`] if deleting fails.
pub fn remove(root: &Path, spec: &TreeSpec) -> Result<(), FixtureError> {
    let tree_dir = root.join(spec.name);
    let manifest = manifest_path(root, spec);
    let partial = partial_marker_path(root, spec);
    if tree_dir.exists() {
        if !manifest.exists() && !partial.exists() {
            return Err(FixtureError::Foreign { path: tree_dir });
        }
        fs::remove_dir_all(&tree_dir).map_err(io_err(&tree_dir))?;
    }
    for marker in [manifest, partial] {
        match fs::remove_file(&marker) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => {
                return Err(FixtureError::Io {
                    path: marker,
                    source: err,
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn manifest_path(root: &Path, spec: &TreeSpec) -> PathBuf {
    root.join(format!("{}.manifest.json", spec.name))
}

fn partial_marker_path(root: &Path, spec: &TreeSpec) -> PathBuf {
    root.join(format!("{}.partial", spec.name))
}

fn io_err(path: &Path) -> impl Fn(io::Error) -> FixtureError {
    let path = path.to_owned();
    move |source| FixtureError::Io {
        path: path.clone(),
        source,
    }
}

fn create_dir(path: &Path) -> Result<(), FixtureError> {
    fs::create_dir_all(path).map_err(io_err(path))
}

fn write_file(
    tree_dir: &Path,
    planned: &PlannedFile,
    buffer: &mut Vec<u8>,
) -> Result<(), FixtureError> {
    let path = tree_dir.join(&planned.rel_path);
    fill_contents(planned, buffer);
    let mut file = File::create(&path).map_err(io_err(&path))?;
    file.write_all(buffer).map_err(io_err(&path))?;
    let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(planned.modified_secs);
    file.set_modified(modified).map_err(io_err(&path))
}

fn fill_contents(planned: &PlannedFile, buffer: &mut Vec<u8>) {
    let size = usize::try_from(planned.size).unwrap_or(usize::MAX);
    let mut rng = Rng::new(planned.content_seed);
    buffer.clear();
    buffer.reserve(size);
    while buffer.len() < size {
        let word = rng.next_u64().to_le_bytes();
        let take = word.len().min(size - buffer.len());
        buffer.extend_from_slice(&word[..take]);
    }
}

fn manifest_for(spec: &TreeSpec, seed: u64, plan: &Plan) -> Manifest {
    let mut extensions = BTreeMap::new();
    for file in &plan.files {
        let name = file.rel_path.rsplit('/').next().unwrap_or_default();
        let extension = match name.rsplit_once('.') {
            Some((stem, ext)) if !stem.is_empty() => ext.to_lowercase(),
            _ => String::new(),
        };
        *extensions.entry(extension).or_insert(0) += 1;
    }
    Manifest {
        name: spec.name.to_owned(),
        generator_version: GENERATOR_VERSION,
        seed,
        files: plan.files.len(),
        dirs: plan.dirs.len(),
        bytes: plan.total_bytes(),
        digest: plan.digest(),
        extensions,
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Order-independent digest of `(relative path, size, modified seconds)`.
fn tree_digest(mut files: Vec<(String, u64, u64)>) -> String {
    files.sort_unstable();
    let mut hash = FNV_OFFSET;
    for (path, size, modified) in &files {
        hash = fnv1a(hash, path.as_bytes());
        hash = fnv1a(hash, &[0]);
        hash = fnv1a(hash, &size.to_le_bytes());
        hash = fnv1a(hash, &modified.to_le_bytes());
    }
    format!("{hash:016x}")
}

/// `SplitMix64`: small, fast, and fixed forever, unlike a library's default RNG,
/// so fixture output can't change under us.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform value in `0..bound`. `bound` must not be 0.
    fn below(&mut self, bound: u64) -> u64 {
        let wide = u128::from(self.next_u64()) * u128::from(bound);
        u64::try_from(wide >> 64).unwrap_or(0)
    }

    fn index(&mut self, len: usize) -> usize {
        usize::try_from(self.below(len as u64)).unwrap_or(0)
    }

    fn pick(&mut self, items: &[&'static str]) -> &'static str {
        items[self.index(items.len())]
    }

    /// Picks an item with probability proportional to its weight.
    fn weighted<T: Copy>(&mut self, table: &[(u32, T)]) -> T {
        let total: u64 = table.iter().map(|(weight, _)| u64::from(*weight)).sum();
        let mut roll = self.below(total);
        for (weight, item) in table {
            if roll < u64::from(*weight) {
                return *item;
            }
            roll -= u64::from(*weight);
        }
        table[0].1
    }
}

struct DirTree {
    /// Index 0 is the root, as the empty string.
    paths: Vec<String>,
}

impl DirTree {
    fn flat() -> Self {
        Self {
            paths: vec![String::new()],
        }
    }

    fn nested(rng: &mut Rng, dirs: usize, max_depth: usize) -> Self {
        let depths: Vec<_> = DEPTH_WEIGHTS
            .iter()
            .copied()
            .filter(|(_, depth)| *depth <= max_depth)
            .collect();
        let mut paths = vec![String::new()];
        // Directory indexes by depth; index 0 of `by_depth` holds only the root.
        let mut by_depth = vec![Vec::new(); max_depth + 1];
        by_depth[0].push(0);
        let mut taken = HashSet::with_capacity(dirs);
        while paths.len() <= dirs {
            let depth = rng.weighted(&depths);
            let candidates = &by_depth[depth - 1];
            if candidates.is_empty() {
                continue; // Nothing that deep yet; depth 1 is always possible.
            }
            let parent = candidates[rng.index(candidates.len())];
            let name = unique_name(
                rng.pick(DIR_NAMES).to_owned(),
                paths.len(),
                parent,
                &mut taken,
            );
            let path = match paths[parent].as_str() {
                "" => name,
                parent_path => format!("{parent_path}/{name}"),
            };
            by_depth[depth].push(paths.len());
            paths.push(path);
        }
        Self { paths }
    }

    /// Picks a directory, skewed so some hold several times more files than others.
    /// Very large single folders are what the flat trees are for.
    fn pick_for_file(&self, rng: &mut Rng) -> usize {
        let a = rng.index(self.paths.len());
        let b = rng.index(self.paths.len());
        let c = rng.index(self.paths.len());
        a.min(b).min(c)
    }
}

/// Keeps `name` as is if it's free in `dir`, otherwise makes it unique with
/// `index`. Compares case-insensitively because NTFS does.
fn unique_name(
    name: String,
    index: usize,
    dir: usize,
    taken: &mut HashSet<(usize, String)>,
) -> String {
    if taken.insert((dir, name.to_lowercase())) {
        return name;
    }
    let unique = match name.split_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem}-{index}.{ext}"),
        _ => format!("{name}-{index}"),
    };
    taken.insert((dir, unique.to_lowercase()));
    unique
}

#[derive(Debug, Clone, Copy)]
enum SizeClass {
    /// Under 4 KiB.
    Tiny,
    /// 4–16 KiB.
    Small,
    /// 16–32 KiB.
    Medium,
    /// 32–128 KiB.
    Large,
    /// 1–4 MiB. Rare: real media files are bigger, but listing only reads
    /// metadata, so bigger files would cost disk space and measure nothing.
    Huge,
}

impl SizeClass {
    fn sample(self, rng: &mut Rng) -> u64 {
        const KIB: u64 = 1024;
        let (low, high) = match self {
            Self::Tiny => (0, 4 * KIB),
            Self::Small => (4 * KIB, 16 * KIB),
            Self::Medium => (16 * KIB, 32 * KIB),
            Self::Large => (32 * KIB, 128 * KIB),
            Self::Huge => (KIB * KIB, 4 * KIB * KIB),
        };
        low + rng.below(high - low)
    }
}

struct GeneratedName {
    name: String,
    size: SizeClass,
}

#[derive(Debug, Clone, Copy)]
enum Category {
    Source,
    Document,
    Photo,
    Screenshot,
    Media,
    Archive,
    Hashed,
    Data,
    NoExtension,
    NonAscii,
}

/// Directories per depth: most sit a few levels down, few at the extremes.
const DEPTH_WEIGHTS: &[(u32, usize)] = &[
    (2, 1),
    (6, 2),
    (12, 3),
    (18, 4),
    (20, 5),
    (17, 6),
    (12, 7),
    (7, 8),
    (4, 9),
    (2, 10),
];

const CATEGORIES: &[(u32, Category)] = &[
    (300, Category::Source),
    (150, Category::Document),
    (130, Category::Photo),
    (40, Category::Screenshot),
    (50, Category::Media),
    (40, Category::Archive),
    (140, Category::Hashed),
    (90, Category::Data),
    (20, Category::NoExtension),
    (40, Category::NonAscii),
];

const SOURCE_SIZES: &[(u32, SizeClass)] = &[
    (75, SizeClass::Tiny),
    (23, SizeClass::Small),
    (2, SizeClass::Medium),
];
const DOCUMENT_SIZES: &[(u32, SizeClass)] = &[
    (20, SizeClass::Tiny),
    (45, SizeClass::Small),
    (30, SizeClass::Medium),
    (5, SizeClass::Large),
];
const PHOTO_SIZES: &[(u32, SizeClass)] = &[
    (35, SizeClass::Small),
    (60, SizeClass::Medium),
    (5, SizeClass::Large),
];
const MEDIA_SIZES: &[(u32, SizeClass)] = &[(995, SizeClass::Large), (5, SizeClass::Huge)];
const ARCHIVE_SIZES: &[(u32, SizeClass)] = &[
    (500, SizeClass::Medium),
    (495, SizeClass::Large),
    (5, SizeClass::Huge),
];
const HASHED_SIZES: &[(u32, SizeClass)] = &[(80, SizeClass::Tiny), (20, SizeClass::Small)];
const TINY_ONLY: &[(u32, SizeClass)] = &[(1, SizeClass::Tiny)];

const WORDS: &[&str] = &[
    "account",
    "alpha",
    "analysis",
    "archive",
    "backup",
    "budget",
    "build",
    "cache",
    "client",
    "config",
    "contract",
    "dashboard",
    "data",
    "design",
    "draft",
    "editor",
    "export",
    "feature",
    "final",
    "garden",
    "handler",
    "holiday",
    "image",
    "index",
    "invoice",
    "kitchen",
    "layout",
    "letter",
    "logo",
    "meeting",
    "module",
    "notes",
    "parser",
    "photo",
    "plan",
    "project",
    "proposal",
    "query",
    "receipt",
    "report",
    "router",
    "sample",
    "schedule",
    "server",
    "session",
    "settings",
    "summary",
    "table",
    "theme",
    "travel",
    "update",
    "utils",
    "vacation",
    "widget",
];
const SOURCE_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx", "rs", "py", "go", "c", "h", "cpp", "java", "cs", "css", "html", "md",
    "sh", "json", "toml", "yaml",
];
const DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "docx", "xlsx", "pptx", "txt", "md", "rtf", "odt"];
const DATA_EXTENSIONS: &[&str] = &["csv", "json", "xml", "log", "sql", "parquet", "db"];
const AUDIO_VIDEO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "mp4", "mkv", "mov"];
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "tar.gz", "7z", "rar", "iso", "msi"];
const HASHED_EXTENSIONS: &[&str] = &["cache", "tmp", "bin", "map", "o", "pack"];
const BARE_NAMES: &[&str] = &[
    "README",
    "LICENSE",
    "Makefile",
    "Dockerfile",
    "CHANGELOG",
    ".gitignore",
    ".editorconfig",
    ".env",
];
const NON_ASCII_STEMS: &[&str] = &[
    "résumé",
    "Übersicht",
    "año nuevo",
    "日本語メモ",
    "会議の議事録",
    "Отчёт за квартал",
    "Привет мир",
    "Σημειώσεις",
    "📁 archive",
    "مستند",
];
const DIR_NAMES: &[&str] = &[
    "src",
    "lib",
    "docs",
    "assets",
    "test",
    "tests",
    "build",
    "dist",
    "vendor",
    "node_modules",
    "images",
    "photos",
    "music",
    "videos",
    "backup",
    "archive",
    "reports",
    "projects",
    "2021",
    "2022",
    "2023",
    "2024",
    "2025",
    "components",
    "utils",
    "core",
    "public",
    "data",
    "config",
    "Downloads",
    "Documents",
    "tmp",
    "old",
    "shared",
];

#[allow(clippy::too_many_lines)] // A flat table of name templates, one arm per category.
fn generate_name(rng: &mut Rng) -> GeneratedName {
    match rng.weighted(CATEGORIES) {
        Category::Source => {
            let extension = rng.pick(SOURCE_EXTENSIONS);
            let name = match rng.below(3) {
                0 => format!(
                    "{}{}.{extension}",
                    rng.pick(WORDS),
                    capitalized(rng.pick(WORDS))
                ),
                1 => format!("{}-{}.test.{extension}", rng.pick(WORDS), rng.pick(WORDS)),
                _ => format!("{}_{}.{extension}", rng.pick(WORDS), rng.pick(WORDS)),
            };
            GeneratedName {
                name,
                size: rng.weighted(SOURCE_SIZES),
            }
        }
        Category::Document => {
            let extension = rng.pick(DOCUMENT_EXTENSIONS);
            let year = 2018 + rng.below(8);
            let name = match rng.below(3) {
                0 => format!(
                    "{} {} {year} (final).{extension}",
                    capitalized(rng.pick(WORDS)),
                    rng.pick(WORDS)
                ),
                1 => format!(
                    "{}_q{}_{year}.{extension}",
                    rng.pick(WORDS),
                    1 + rng.below(4)
                ),
                _ => format!(
                    "{} - {} v{}.{extension}",
                    capitalized(rng.pick(WORDS)),
                    capitalized(rng.pick(WORDS)),
                    1 + rng.below(12)
                ),
            };
            GeneratedName {
                name,
                size: rng.weighted(DOCUMENT_SIZES),
            }
        }
        Category::Photo => {
            let name = if rng.below(2) == 0 {
                format!(
                    "IMG_{}{:02}{:02}_{:06}.jpg",
                    2018 + rng.below(8),
                    1 + rng.below(12),
                    1 + rng.below(28),
                    rng.below(240_000)
                )
            } else {
                format!(
                    "DSC{:05}.{}",
                    rng.below(100_000),
                    rng.pick(&["JPG", "NEF", "heic"])
                )
            };
            GeneratedName {
                name,
                size: rng.weighted(PHOTO_SIZES),
            }
        }
        Category::Screenshot => GeneratedName {
            name: format!(
                "Screenshot {}-{:02}-{:02} {:06}.png",
                2018 + rng.below(8),
                1 + rng.below(12),
                1 + rng.below(28),
                rng.below(240_000)
            ),
            size: rng.weighted(PHOTO_SIZES),
        },
        Category::Media => GeneratedName {
            name: format!(
                "{} {} - {:02}.{}",
                capitalized(rng.pick(WORDS)),
                rng.pick(WORDS),
                1 + rng.below(40),
                rng.pick(AUDIO_VIDEO_EXTENSIONS)
            ),
            size: rng.weighted(MEDIA_SIZES),
        },
        Category::Archive => GeneratedName {
            name: format!(
                "{}-{}.{}.{}.{}",
                rng.pick(WORDS),
                rng.below(10),
                rng.below(20),
                rng.below(50),
                rng.pick(ARCHIVE_EXTENSIONS)
            ),
            size: rng.weighted(ARCHIVE_SIZES),
        },
        Category::Hashed => GeneratedName {
            name: format!("{:016x}.{}", rng.next_u64(), rng.pick(HASHED_EXTENSIONS)),
            size: rng.weighted(HASHED_SIZES),
        },
        Category::Data => GeneratedName {
            name: format!(
                "{}_{}_{:04}.{}",
                rng.pick(WORDS),
                rng.pick(WORDS),
                rng.below(10_000),
                rng.pick(DATA_EXTENSIONS)
            ),
            size: rng.weighted(DOCUMENT_SIZES),
        },
        Category::NoExtension => GeneratedName {
            name: rng.pick(BARE_NAMES).to_owned(),
            size: rng.weighted(TINY_ONLY),
        },
        Category::NonAscii => GeneratedName {
            name: format!(
                "{} {}.{}",
                rng.pick(NON_ASCII_STEMS),
                rng.below(1000),
                rng.pick(DOCUMENT_EXTENSIONS)
            ),
            size: rng.weighted(DOCUMENT_SIZES),
        },
    }
}

fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_NESTED: TreeSpec = TreeSpec {
        name: "test-nested",
        files: 400,
        shape: Shape::Nested {
            dirs: 30,
            max_depth: 4,
        },
    };

    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new(test: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("vault-fixtures-{test}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn same_seed_gives_same_plan_and_other_seed_differs() {
        assert_eq!(plan(&SMALL_NESTED, 1), plan(&SMALL_NESTED, 1));
        assert_ne!(
            plan(&SMALL_NESTED, 1).digest(),
            plan(&SMALL_NESTED, 2).digest()
        );
    }

    /// Fails if the output changes without a [`GENERATOR_VERSION`] bump, and,
    /// because CI runs it on Windows, if the output ever differs across OSes.
    #[test]
    fn golden_digests() {
        assert_eq!(GENERATOR_VERSION, 1);
        assert_eq!(plan(&FLAT_10K, DEFAULT_SEED).digest(), "701715e1c8d29f7e");
        assert_eq!(
            plan(&SMALL_NESTED, DEFAULT_SEED).digest(),
            "b5ec445cb55d9cce"
        );
    }

    #[test]
    fn flat_trees_have_exact_counts_and_no_directories() {
        let plan = plan(&FLAT_10K, DEFAULT_SEED);
        assert_eq!(plan.files.len(), 10_000);
        assert_eq!(plan.dirs.len(), 0);
        assert!(plan.files.iter().all(|f| !f.rel_path.contains('/')));
    }

    #[test]
    fn names_are_unique_ignoring_case_and_valid_on_windows() {
        const RESERVED: [&str; 4] = ["con", "prn", "aux", "nul"];
        let plan = plan(&FLAT_100K, DEFAULT_SEED);
        let mut seen = HashSet::new();
        for file in &plan.files {
            let name = &file.rel_path;
            assert!(seen.insert(name.to_lowercase()), "duplicate: {name}");
            assert!(
                !name.contains(['<', '>', ':', '"', '\\', '|', '?', '*']),
                "{name}"
            );
            assert!(!name.ends_with(['.', ' ']), "{name}");
            let stem = name.split('.').next().unwrap().to_lowercase();
            assert!(!RESERVED.contains(&stem.as_str()), "{name}");
        }
    }

    #[test]
    fn mix_is_realistic() {
        let plan = plan(&FLAT_100K, DEFAULT_SEED);
        let manifest = manifest_for(&FLAT_100K, DEFAULT_SEED, &plan);
        assert!(manifest.extensions.len() > 40, "{:?}", manifest.extensions);
        assert!(plan.files.iter().any(|f| !f.rel_path.is_ascii()));
        assert!(plan.files.iter().any(|f| f.rel_path.contains(' ')));
        assert!(plan.files.iter().any(|f| f.size < 1024));
        assert!(plan.files.iter().any(|f| f.size > 1024 * 1024));
    }

    #[test]
    fn nested_tree_respects_shape_and_stays_under_max_path() {
        let plan = plan(&NESTED_1M, DEFAULT_SEED);
        assert_eq!(plan.files.len(), 1_000_000);
        assert_eq!(plan.dirs.len(), 50_000);
        let depth = |path: &str| path.matches('/').count();
        let deepest_dir = plan.dirs.iter().map(|d| depth(d) + 1).max().unwrap();
        assert_eq!(deepest_dir, 10);
        let mut file_depths: Vec<_> = plan.files.iter().map(|f| depth(&f.rel_path)).collect();
        file_depths.sort_unstable();
        let median_depth = file_depths[file_depths.len() / 2];
        assert!(
            (3..=7).contains(&median_depth),
            "median depth {median_depth}"
        );
        assert!(file_depths[file_depths.len() - 1] >= 8);
        let unique: HashSet<_> = plan
            .files
            .iter()
            .map(|f| f.rel_path.to_lowercase())
            .collect();
        assert_eq!(unique.len(), plan.files.len());
        let longest = plan.files.iter().map(|f| f.rel_path.len()).max().unwrap();
        assert!(longest < 200, "longest path is {longest} bytes");
    }

    #[test]
    fn generated_tree_is_byte_identical_for_the_same_seed() {
        let (a, b) = (TempRoot::new("identical-a"), TempRoot::new("identical-b"));
        let manifest_a = generate(&a.0, &SMALL_NESTED, 7, false).unwrap();
        let manifest_b = generate(&b.0, &SMALL_NESTED, 7, false).unwrap();
        assert_eq!(manifest_a, manifest_b);
        for file in plan(&SMALL_NESTED, 7).files {
            let in_a = fs::read(a.0.join(SMALL_NESTED.name).join(&file.rel_path)).unwrap();
            let in_b = fs::read(b.0.join(SMALL_NESTED.name).join(&file.rel_path)).unwrap();
            assert_eq!(in_a, in_b, "{}", file.rel_path);
            assert_eq!(in_a.len() as u64, file.size);
        }
    }

    #[test]
    fn verify_accepts_a_good_tree_and_catches_changes() {
        let root = TempRoot::new("verify");
        generate(&root.0, &SMALL_NESTED, 7, false).unwrap();
        verify(&root.0, &SMALL_NESTED, 7).unwrap();

        let extra = root.0.join(SMALL_NESTED.name).join("extra.txt");
        fs::write(&extra, "x").unwrap();
        assert!(matches!(
            verify(&root.0, &SMALL_NESTED, 7),
            Err(FixtureError::Mismatch { .. })
        ));
        fs::remove_file(&extra).unwrap();

        let first = &plan(&SMALL_NESTED, 7).files[0];
        let path = root.0.join(SMALL_NESTED.name).join(&first.rel_path);
        fs::write(&path, vec![0; usize::try_from(first.size).unwrap() + 1]).unwrap();
        assert!(matches!(
            verify(&root.0, &SMALL_NESTED, 7),
            Err(FixtureError::Mismatch { .. })
        ));
    }

    #[test]
    fn require_rejects_missing_tree_and_other_seed() {
        let root = TempRoot::new("require");
        assert!(matches!(
            require(&root.0, &SMALL_NESTED, 7),
            Err(FixtureError::NotGenerated { .. })
        ));
        generate(&root.0, &SMALL_NESTED, 7, false).unwrap();
        assert!(require(&root.0, &SMALL_NESTED, 7).is_ok());
        assert!(matches!(
            require(&root.0, &SMALL_NESTED, 8),
            Err(FixtureError::NotGenerated { .. })
        ));
    }

    #[test]
    fn generate_replaces_an_interrupted_tree() {
        let root = TempRoot::new("interrupted");
        generate(&root.0, &SMALL_NESTED, 7, false).unwrap();
        let tree = root.0.join(SMALL_NESTED.name);
        fs::write(tree.join("leftover.txt"), "x").unwrap();
        fs::write(partial_marker_path(&root.0, &SMALL_NESTED), "").unwrap();

        assert!(require(&root.0, &SMALL_NESTED, 7).is_err());
        generate(&root.0, &SMALL_NESTED, 7, false).unwrap();
        verify(&root.0, &SMALL_NESTED, 7).unwrap();
        assert!(!partial_marker_path(&root.0, &SMALL_NESTED).exists());
    }

    #[test]
    fn never_deletes_a_directory_it_did_not_create() {
        let root = TempRoot::new("foreign");
        let tree = root.0.join(SMALL_NESTED.name);
        fs::create_dir(&tree).unwrap();
        fs::write(tree.join("precious.txt"), "keep me").unwrap();

        assert!(matches!(
            generate(&root.0, &SMALL_NESTED, 7, true),
            Err(FixtureError::Foreign { .. })
        ));
        assert!(matches!(
            remove(&root.0, &SMALL_NESTED),
            Err(FixtureError::Foreign { .. })
        ));
        assert_eq!(
            fs::read_to_string(tree.join("precious.txt")).unwrap(),
            "keep me"
        );
    }

    #[test]
    fn name_corpus_is_deterministic_and_sized() {
        let names = name_corpus(1000, 3);
        assert_eq!(names.len(), 1000);
        assert_eq!(names, name_corpus(1000, 3));
    }
}
