use crate::cmp::Ord;
use crate::option::Option;
use crate::convert::From;

impl<T, let N: u32> [T; N] {
    /// Returns the length of the slice.
    #[builtin(array_len)]
    pub fn len(self) -> u32 {}

    pub fn sort(self) -> Self where T: Ord {
        self.sort_via(|a: T, b: T| a <= b)
    }

    pub fn sort_via<Env>(self, ordering: fn[Env](T, T) -> bool) -> Self {
        let sorted_index = unsafe {
            // Safety: These indices are asserted to be the sorted element indices via `find_index`
            let sorted_index: [u32; N] = self.get_sorting_index(ordering);

            for i in 0..N {
                let pos = find_index(sorted_index, i);
                assert(sorted_index[pos] == i);
            }

            sorted_index
        };

        // Sort the array using the indexes
        let mut result = self;
        for i in 0..N {
            result[i] = self[sorted_index[i]];
        }
        // Ensure the array is sorted
        for i in 0..N - 1 {
            assert(ordering(result[i], result[i + 1]));
        }

        result
    }

    /// Returns the index of the elements in the array that would sort it, using the provided custom sorting function.
    unconstrained fn get_sorting_index<Env>(self, ordering: fn[Env](T, T) -> bool) -> [u32; N] {
        let mut result = [0; N];
        let mut a = self;
        for i in 0..N {
            result[i] = i;
        }
        for i in 1..N {
            for j in 0..i {
                if ordering(a[i], a[j]) {
                    let old_a_j = a[j];
                    a[j] = a[i];
                    a[i] = old_a_j;
                    let old_j = result[j];
                    result[j] = result[i];
                    result[i] = old_j;
                }
            }
        }
        result
    }

    #[builtin(as_slice)]
    pub fn as_slice(self) -> [T] {}

    // Apply a function to each element of an array, returning a new array
    // containing the mapped elements.
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> [U; N] {
        let first_elem = f(self[0]);
        let mut ret = [first_elem; N];

        for i in 1..self.len() {
            ret[i] = f(self[i]);
        }

        ret
    }

    // Apply a function to each element of the array and an accumulator value,
    // returning the final accumulated value. This function is also sometimes
    // called `foldl`, `fold_left`, `reduce`, or `inject`.
    pub fn fold<U, Env>(self, mut accumulator: U, f: fn[Env](U, T) -> U) -> U {
        for elem in self {
            accumulator = f(accumulator, elem);
        }
        accumulator
    }

    // Apply a function to each element of the array and an accumulator value,
    // returning the final accumulated value. Unlike fold, reduce uses the first
    // element of the given array as its starting accumulator value.
    pub fn reduce<Env>(self, f: fn[Env](T, T) -> T) -> T {
        let mut accumulator = self[0];
        for i in 1..self.len() {
            accumulator = f(accumulator, self[i]);
        }
        accumulator
    }

    // Returns true if all elements in the array satisfy the predicate
    pub fn all<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = true;
        for elem in self {
            ret &= predicate(elem);
        }
        ret
    }

    // Returns true if any element in the array satisfies the predicate
    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        for elem in self {
            ret |= predicate(elem);
        }
        ret
    }
}

impl<let N: u32> [u8; N] {
    /// Convert a sequence of bytes as-is into a string.
    /// This function performs no UTF-8 validation or similar.
    #[builtin(array_as_str_unchecked)]
    pub fn as_str_unchecked(self) -> str<N> {}
}

// helper function used to look up the position of a value in an array of Field
// Note that function returns 0 if the value is not found
unconstrained fn find_index<let N: u32>(a: [u32; N], find: u32) -> u32 {
    let mut result = 0;
    for i in 0..a.len() {
        if a[i] == find {
            result = i;
        }
    }
    result
}

impl<let N: u32> From<str<N>> for [u8; N] {
    fn from(s: str<N>) -> Self {
        s.as_bytes()
    }
}
