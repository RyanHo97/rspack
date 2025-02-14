use derivative::Derivative;
use rspack_core::{ChunkUkey, Module};
use rspack_identifier::IdentifierSet;
use rustc_hash::FxHashSet;

use crate::common::SplitChunkSizes;

/// `ModuleGroup` is a abstraction of middle step for splitting chunks.
///
/// `ModuleGroup` captures/contains a bunch of modules due to the `optimization.splitChunks` configuration.
///
/// `ModuleGroup` would be transform into `Chunk` in the end.
///
/// The original name of `ModuleGroup` is `ChunkInfoItem` borrowed from Webpack
#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct ModuleGroup {
  #[derivative(Debug = "ignore")]
  pub modules: IdentifierSet,
  pub cache_group_index: usize,
  pub cache_group_priority: f64,
  pub name: String,
  pub sizes: SplitChunkSizes,
  #[derivative(Debug = "ignore")]
  pub chunks: FxHashSet<ChunkUkey>,
}

impl ModuleGroup {
  pub fn add_module(&mut self, module: &dyn Module) {
    let old_len = self.modules.len();
    self.modules.insert(module.identifier());

    if self.modules.len() != old_len {
      module.source_types().iter().for_each(|ty| {
        let size = self.sizes.entry(*ty).or_default();
        *size += module.size(ty);
      });
    }
  }

  pub fn remove_module(&mut self, module: &dyn Module) {
    let old_len = self.modules.len();
    self.modules.remove(&module.identifier());

    if self.modules.len() != old_len {
      module.source_types().iter().for_each(|ty| {
        let size = self.sizes.entry(*ty).or_default();
        *size -= module.size(ty);
        *size = size.max(0.0)
      });
    }
  }
}

pub(crate) fn compare_entries(a: &ModuleGroup, b: &ModuleGroup) -> f64 {
  // 1. by priority
  let diff_priority = a.cache_group_priority - b.cache_group_priority;
  if diff_priority != 0f64 {
    return diff_priority;
  }
  // 2. by number of chunks
  let diff_count = a.chunks.len() as f64 - b.chunks.len() as f64;
  if diff_count != 0f64 {
    return diff_count;
  }

  // // 3. by size reduction
  // let a_size_reduce = total_size(&a.sizes) * (a.chunks.len() - 1) as f64;
  // let b_size_reduce = total_size(&b.sizes) * (b.chunks.len() - 1) as f64;
  // let diff_size_reduce = a_size_reduce - b_size_reduce;
  // if diff_size_reduce != 0f64 {
  //   return diff_size_reduce;
  // }
  // 4. by cache group index
  let index_diff = b.cache_group_index as f64 - a.cache_group_index as f64;
  if index_diff != 0f64 {
    return index_diff;
  }

  // 5. by number of modules (to be able to compare by identifier)
  let modules_a_len = a.modules.len();
  let modules_b_len = b.modules.len();
  let diff = modules_a_len as f64 - modules_b_len as f64;
  if diff != 0f64 {
    return diff;
  }

  let mut modules_a = a.modules.iter().collect::<Vec<_>>();
  let mut modules_b = b.modules.iter().collect::<Vec<_>>();
  modules_a.sort_unstable();
  modules_b.sort_unstable();
  modules_a.cmp(&modules_b) as usize as f64
}
