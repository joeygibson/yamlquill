//! Abstract syntax tree types for YAMLPath expressions.

/// A segment in a YAMLPath expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// Root node ($)
    Root,
    /// Current node (@) - reserved for future filter support
    Current,
    /// Named child (.property or ['property'])
    Child(String),
    /// Array index ([0], [-1])
    Index(isize),
    /// Wildcard (* or [*]) - all children
    Wildcard,
    /// Recursive descent (.. or ..property)
    RecursiveDescent(Option<String>),
    /// Array slice ([start:end])
    Slice(Option<isize>, Option<isize>),
    /// Multiple properties (['prop1','prop2'])
    MultiProperty(Vec<String>),
}

/// A complete YAMLPath expression.
#[derive(Debug, Clone, PartialEq)]
pub struct YamlPath {
    /// Segments that make up the path.
    pub segments: Vec<PathSegment>,
}

impl YamlPath {
    /// Creates a new YAMLPath with the given segments.
    pub fn new(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }
}
