pub mod instruction;
pub mod processor;
pub mod state;


#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;



// ++++++++++++++++ Learnings ++++++++++++++++
// - each program is processed by its BPF Loader and has an entrypoint whose      
//   structure depends on which BPF Loader is used.
