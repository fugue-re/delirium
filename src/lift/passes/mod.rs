/// Transformation passes applied to lifted ECode to obtain our
/// representation.
/// 
/// These passes remove Ghidra's quirks and regularise the IR/IL we
/// emit.
/// 
/// - We do away with Ghidra's notion of the unique space for
///   temporaries and make each temporary unique.
/// 
/// - We replace all aliased variables, including register views,
///   such as AL, AH, AX when we are dealing with x86, with the
///   base register, e.g., EAX or RAX.

mod aliases;
mod temporaries;

mod visit;
mod visit_mut;