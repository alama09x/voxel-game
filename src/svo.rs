// SVO Implementation
// May come in handy at some point
// https://gist.github.com/Eisenwave/c48bf988fc29d1c8bf0d4512d3916d22

#[derive(Debug, Default, Clone)]
struct Svo<V: Voxel> {
    root: SvoNode<V>,
    depth: u32,
}

#[derive(Debug, Clone)]
enum SvoNode<V: Voxel> {
    Leaf([V; 8]),
    Branch([Option<Box<Self>>; 8]),
}

impl<V: Voxel> Default for SvoNode<V> {
    fn default() -> Self {
        Self::Leaf([V::default(); 8])
    }
}

impl<V: Voxel> Svo<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pos: [i32; 3], voxel: V) {
        self.ensure_space(pos);
        self.insert_internal(self.index_of(pos), voxel);
    }

    pub fn get(&self, pos: [i32; 3]) -> Option<&V> {
        self.find(self.index_of(pos))
    }

    pub fn get_or_create(&mut self, pos: [i32; 3]) -> &mut V {
        self.ensure_space(pos);
        self.find_or_create(self.index_of(pos))
    }

    fn find_or_create(&mut self, node_index: u64) -> &mut V {
        let mut current = &mut self.root;

        // Current bit shift
        let mut s = self.depth * 3;

        while s >= 3 {
            let oct_digit = ((node_index >> s) & 0b111) as usize;

            match current {
                SvoNode::Branch(children) => {
                    if children[oct_digit].is_none() {
                        let new_node = Box::new(if s == 3 {
                            SvoNode::Leaf([V::default(); 8])
                        } else {
                            SvoNode::Branch([const { None }; 8])
                        });
                        children[oct_digit] = Some(new_node);
                    }
                    current = children[oct_digit].as_mut().unwrap();
                }
                SvoNode::Leaf(data) => {
                    return &mut data[oct_digit];
                }
            }
            s -= 3;
        }
        unreachable!()
    }

    fn find(&self, node_index: u64) -> Option<&V> {
        let mut current = &self.root;
        let mut s = self.depth * 3;

        while s >= 3 {
            let oct_digit = ((node_index >> s) & 0b11) as usize;

            match current {
                SvoNode::Branch(children) => {
                    if children[oct_digit].is_none() {
                        return None;
                    }
                    current = children[oct_digit].as_ref().unwrap();
                }
                SvoNode::Leaf(data) => {
                    return Some(&data[oct_digit]);
                }
            }
            s -= 3;
        }
        unreachable!()
    }

    fn min_include(&self) -> i32 {
        -(1 << self.depth)
    }

    fn max_include(&self) -> i32 {
        (1 << self.depth) - 1
    }

    fn min_exclude(&self) -> i32 {
        -(1 << self.depth) - 1
    }

    fn max_exclude(&self) -> i32 {
        1 << self.depth
    }

    fn index_of(&self, pos: [i32; 3]) -> u64 {
        let upos = pos.map(|i| (i - self.min_include()) as u32);
        bits::ileave3(upos)
    }

    fn ensure_space(&mut self, pos: [i32; 3]) {
        let limit = self.bounds_test(pos);
        if limit != 0 {
            self.grow(limit);
        }
    }

    fn insert_internal(&mut self, node_index: u64, voxel: V) {
        *self.find_or_create(node_index) = voxel;
    }

    fn grow(&mut self, limit: u32) {
        let mut size = 1;
        while size <= limit {
            self.grow_once();
            self.depth <<= 1;
            size = 1 << self.depth;
        }
    }

    fn grow_once(&mut self) {
        for i in 0..8 {
            if let SvoNode::Branch(children) = &mut self.root {
                if children[i].is_none() {
                    continue;
                }
                let mut siblings = [const { None }; 8];
                siblings[!i & 0b111] = children[i].clone();

                let parent = Box::new(SvoNode::Branch(siblings));
                children[i] = Some(parent);
            }
        }
    }

    fn bounds_test(&self, pos: [i32; 3]) -> u32 {
        fn abs_for_bounds_test(x: i32) -> u32 {
            (if x < 0 { -x - 1 } else { x }) as u32
        }
        assert_eq!(abs_for_bounds_test(-5), 4);
        let max =
            abs_for_bounds_test(pos[0]) | abs_for_bounds_test(pos[1]) | abs_for_bounds_test(pos[2]);

        if max >= (1u32 << self.depth) {
            max
        } else {
            0
        }
    }
}

mod bits {
    #[bitmatch::bitmatch]
    pub fn ileave3([x, y, z]: [u32; 3]) -> u64 {
        (bitpack!(
            "xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz_xyz"
        )) as u64
    }
}
