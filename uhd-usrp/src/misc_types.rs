
// macro_rules! gen_handle_struct_wrapper {
//     ($name:ident, $ty:ty, $alloc:path, $free:path) => {
//         pub(crate) struct $name {
//             handle: $ty,
//         }

//         impl $name {
//             pub(crate) fn handle(&self) -> &$ty {
//                 &self.handle
//             }

//             pub(crate) fn handle_mut(&mut self) -> &mut $ty {
//                 &mut self.handle
//             }
//         }

//         impl Drop for $name {
//             fn drop(&mut self) {
//                 unsafe {
//                     $free(std::ptr::addr_of_mut!(self.handle));
//                 }
//             }
//         }
//     };
// }
// pub(crate) use gen_handle_struct_wrapper;

// macro_rules! gen_handle_struct_getter {
//     ($name:ident, $ty:ty, $alloc:path, $free:path) => {};
// }

// macro_rules! gen_handle_struct_getter {
//     (fn $name:ident(&self) -> Result<$ty:ty> { $internal:path }) => {
//         fn $name(&self) -> crate::error::Result<$ty> {
//             unsafe {
//                 let mut result = std::mem::MaybeUninit::uninit();
//                 crate::error::try_uhd!($internal(self.handle, result.as_mut_ptr()))
//                     .and_then(|_| Ok(result.assume_init()))
//             }
//         }
//     };
//     (fn $name:ident(&self, $($a:ident: $t:ty),*) -> Result<$ty:ty> { $internal:path => ($($v2:expr),*, _) }) => {
//         fn $name(&self, $($a: $t),*) -> crate::error::Result<$ty> {
//             unsafe {
//                 let mut result = std::mem::MaybeUninit::uninit();
//                 crate::error::try_uhd!($internal(self.handle, $($v2),*, result.as_mut_ptr()))
//                 .and_then(|_| Ok(result.assume_init()))
//             }
//         }
//     };
// }
// pub(crate) use gen_handle_struct_getter;
