# Changelog

## [0.1.5] - 2024-10-06

### Changed
- `SliceOwner` no longer implements `IntoIterator`

## [0.1.4] - 2024-10-06

### Added
- `SliceOwner` implementation for `Box<T>`

## [0.1.3] - 2024-10-06

### Added
- Documentation link.
### Removed
- `slice::owner::ops` mod has been removed since I forgot it last update. It was an empty blanked implementation so it's not relevant.

## [0.1.2] - 2024-10-06

### Added
- Implemented additional traits for `StackVec<T, N>`.
- Implemented calls for `traits::Vec` methods from `StackVec<T, N>` in order to not need more to import `traits::Vec` while using only `StackVec<T, N>`.

### Removed
- `slice::owner::ops` mod has been removed since it's useless. `SliceOwner::as_slice` provides direct access to the inner slice. `SliceOwner` implementations will commonly implement `Deref<Target = [<SliceOwner as IntoIterator>::Item]` and `DerefMut` in order to access slice methods directly.

## [0.1.1] - 2024-10-01

### Added
- CHANGELOG
- Implemented additional traits for `StackVec<T, N>`.

### Fixed
- `StackVec<T, N>::capacity()` now correctly handles `usize::MAX` for zero-sized `T` types..

## [0.1.0] - 2024-09-30

### Added
- Initial release.
- Basic implementation of resizable arrays for std and no_std environments.