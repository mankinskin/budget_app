use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        PatternId,
        Pattern,
        PatternView,
        TokenPosition,
        Child,
        index_splitter::{
            IndexSplitter,
            IndexInParent,
            SplitKey,
            SplitContext,
            IndexSplit,
            PatternSplit,
        },
        vertex::*,
        prefix,
        postfix,
    },
    token::{
        Tokenize,
    },
};
use std::{
    num::NonZeroUsize,
};

pub type Split = (Pattern, Pattern);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SplitIndex {
    pos: TokenPosition,
    index: VertexIndex,
    index_pos: IndexPosition,
}
impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't + std::fmt::Display,
{
    /// Split a pattern before the specified index
    pub fn split_pattern_at_index(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> (Pattern, Pattern) {
        (
            prefix(pattern, index),
            postfix(pattern, index)
        )
    }
    pub fn split_context(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> (Pattern, Pattern) {
        (
            prefix(pattern, index),
            postfix(pattern, index+1)
        )
    }
    /// find split position in index in pattern
    pub fn find_pattern_split_index(
        pattern: impl Iterator<Item=Child>,
        pos: NonZeroUsize,
        ) -> Option<SplitIndex> {
        let mut skipped = 0;
        let pos: TokenPosition = pos.into();
        // find child overlapping with cut pos or 
        pattern.enumerate()
            .find_map(|(i, child)| {
                if skipped + child.get_width() <= pos {
                    skipped += child.get_width();
                    None
                } else {
                    Some(SplitIndex {
                        index_pos: i,
                        pos: pos - skipped,
                        index: child.index
                    })
                }
            })
    }
    /// Find split indicies and positions of multiple patterns
    pub fn find_child_pattern_split_indices(
        patterns: impl Iterator<Item=(PatternId, impl Iterator<Item=Child>)>,
        pos: NonZeroUsize,
        ) -> impl Iterator<Item=(PatternId, SplitIndex)> {
        patterns.filter_map(move |(i, pattern)|
            Self::find_pattern_split_index(pattern, pos).map(|split| (i, split))
        )
    }

    /// search for a perfect split in the split indices (offset = 0)
    fn perfect_split_search(current_node: &'a VertexData, split_indices: impl Iterator<Item=(PatternId, SplitIndex)> + 'a) -> impl Iterator<Item=Result<SplitContext, (Pattern, Pattern)>> + 'a {
        split_indices.map(|(pattern_index, SplitIndex { index_pos, pos, index })| {
            let index_in_parent = IndexInParent {
                pattern_index,
                replaced_index: index_pos,
            };
            NonZeroUsize::new(pos)
                .map(|offset| (
                    SplitKey::new(
                        index,
                        offset,
                    ),
                    index_in_parent.clone()
                ))
                .ok_or(index_in_parent)
        })
        .map(move |r|
            r.map(move |(key, IndexInParent {
                    pattern_index,
                    replaced_index: split_index,
                })| {
                let pattern = current_node.get_child_pattern(&pattern_index).unwrap();
                SplitContext {
                    context: Self::split_context(pattern, split_index),
                    key,
                }
            })
            .map_err(|IndexInParent {
                    pattern_index,
                    replaced_index: split_index,
                }| {
                let pattern = current_node.get_child_pattern(&pattern_index).unwrap();
                Self::split_pattern_at_index(pattern, split_index)
            })
        )
    }
    fn find_perfect_split(current_node: &VertexData, split_indices: impl Iterator<Item=(PatternId, SplitIndex)>) -> Result<(Pattern, Pattern), Vec<SplitContext>> {
        match Self::perfect_split_search(current_node, split_indices).collect() {
            Ok(v) => Err(v),
            Err(i) => Ok(i),
        }
    }
    /// Get perfect split if it exists and remaining pattern split contexts
    pub(crate) fn separate_perfect_split(&self, root: impl Indexed, pos: NonZeroUsize) -> (Option<(Pattern, Pattern)>, Vec<SplitContext>) {
        let current_node = self.get_vertex_data(root).unwrap();
        let children = current_node.get_children().clone();
        let len = children.len();
        let child_slices = children.into_iter().map(|(i, p)| (i, p.into_iter()));
        let split_indices = Self::find_child_pattern_split_indices(child_slices, pos);
        Self::perfect_split_search(current_node, split_indices)
            .fold((None, Vec::with_capacity(len)),
                |(pa, mut sa), r| {
                    match r {
                        Ok(s) => {
                            sa.push(s);
                            (pa, sa)
                        },
                        Err(p) => (Some(p), sa),
                    }
                }
            )
    }

    /// Get perfect split or pattern split contexts
    pub(crate) fn try_perfect_split(&self, root: impl Indexed, pos: NonZeroUsize) -> Result<IndexSplit, Vec<SplitContext>> {
        let current_node = self.get_vertex_data(root).unwrap();
        let children = current_node.get_children().clone();
        let child_slices = children.into_iter().map(|(i, p)| (i, p.into_iter()));
        let split_indices = Self::find_child_pattern_split_indices(child_slices, pos);
        Self::find_perfect_split(current_node, split_indices)
            .map(IndexSplit::from)
    }

    /// Split an index the specified position
    pub fn split_index_at_pos(&mut self, root: impl Indexed, pos: NonZeroUsize) -> (Vec<Pattern>, Vec<Pattern>) {
        // if perfect split on first level (no surrounding context), only return that
        // otherwise build a merge graph over all children until perfect splits are found in all branches
        // then merge patterns upwards into their parent contexts

        IndexSplitter::split(self, root, pos)
        //println!("Split: {} at {} =>", self.index_string(root), pos);
        //println!("left:\n\t{}", self.separated_pattern_string(&left));
        //println!("right:\n\t{}", self.separated_pattern_string(&right));
    }
    fn pattern_split_string_width(&self, split: &PatternSplit) -> usize {
        let left = self.pattern_string_with_separator(&split.prefix, ".");
        let right = self.pattern_string_with_separator(&split.postfix, ".");
        left.len()
        + self.index_split_string_width(&split.inner)
        + right.len()
    }
    fn index_split_string_width(&self, split: &IndexSplit) -> usize {
        split.splits.first().map(|s| self.pattern_split_string_width(s)).unwrap_or(1)
    }
    fn pattern_split_string_with_offset_and_width(&self, split: &PatternSplit, offset: usize, width: usize) -> String{
        let left = self.pattern_string_with_separator(&split.prefix, ".");
        let right = self.pattern_string_with_separator(&split.postfix, ".");
        let next_width = (width - left.len()) - right.len();
        let next_offset = offset + left.len();

        let inner = split
            .inner.splits.iter()
            .fold(String::new(), |acc, s|
                format!("{}{}\n",
                    acc,
                    self.pattern_split_string_with_offset_and_width(
                        s, next_offset, next_width,
                    ),
                )
            );
        format!("{}\n{}", std::iter::repeat(' ').take(offset)
            .chain(left.chars())
            .chain(std::iter::repeat(' ').take(next_width + 1))
            .chain(right.chars())
            .collect::<String>(),
            inner,
        )
    }
    fn index_split_string(&self, split: &IndexSplit) -> String{
        let width = self.index_split_string_width(split);
        split.splits.iter()
        .fold(String::new(), |acc, split|
            format!("{}{}\n", acc,
                self.pattern_split_string_with_offset_and_width(split, 0, width),
            )
        )
    }
    pub fn print_index_split(&self, split: &IndexSplit) {
        print!("{}", self.index_split_string(split));
    }
}
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::hypergraph::{
//        tests::context_mut,
//        child_strings::*,
//    };
//    use pretty_assertions::assert_eq;
//    #[test]
//    fn split_index_1() {
//        let (
//            graph,
//            _a,
//            _b,
//            c,
//            _d,
//            _e,
//            _f,
//            _g,
//            _h,
//            _i,
//            ab,
//            _bc,
//            _cd,
//            _bcd,
//            abc,
//            _abcd,
//            _cdef,
//            _efghi,
//            _abab,
//            _ababab,
//            _ababababcdefghi,
//            ) = &mut *context_mut();
//        let (left, right) = graph.split_index_at_pos(*abc, NonZeroUsize::new(2).unwrap());
//        assert_eq!(left, vec![Child::new(*ab, 2)], "left");
//        assert_eq!(right, vec![Child::new(*c, 1)], "right");
//    }
//    #[test]
//    fn split_child_patterns_2() {
//        let (
//            graph,
//            _a,
//            _b,
//            _c,
//            d,
//            _e,
//            _f,
//            _g,
//            _h,
//            _i,
//            _ab,
//            _bc,
//            _cd,
//            _bcd,
//            abc,
//            abcd,
//            _cdef,
//            _efghi,
//            _abab,
//            _ababab,
//            _ababababcdefghi,
//            ) = &mut *context_mut();
//        let (left, right) = graph.split_index_at_pos(*abcd, NonZeroUsize::new(3).unwrap());
//        assert_eq!(left, vec![Child::new(*abc, 3)], "left");
//        assert_eq!(right, vec![Child::new(*d, 1)], "right");
//    }
//    use crate::token::*;
//    fn split_child_patterns_3_impl() {
//        let mut graph = Hypergraph::default();
//        if let [a, b, c, d] = graph.insert_tokens(
//            [
//                Token::Element('a'),
//                Token::Element('b'),
//                Token::Element('c'),
//                Token::Element('d'),
//            ])[..] {
//            // wxabyzabbyxabyz
//            let ab = graph.insert_pattern([a, b]);
//            let bc = graph.insert_pattern([b, c]);
//            let cd = graph.insert_pattern([c, d]);
//            let abc = graph.insert_patterns([
//                vec![ab, c],
//                vec![a, bc]
//            ]);
//            let bcd = graph.insert_patterns([
//                vec![bc, d],
//                vec![b, cd]
//            ]);
//            let abcd = graph.insert_patterns([
//                vec![abc, d],
//                vec![a, bcd]
//            ]);
//            let ab_pattern = vec![Child::new(ab, 2)];
//            let cd_pattern = vec![Child::new(cd, 2)];
//
//            let (left, right) = graph.split_index_at_pos(abcd, NonZeroUsize::new(2).unwrap());
//            assert_eq!(left, ab_pattern, "left");
//            assert_eq!(right, cd_pattern, "right");
//        } else {
//            panic!();
//        }
//    }
//    #[test]
//    fn split_child_patterns_3() {
//        split_child_patterns_3_impl()
//    }
//    #[test]
//    fn split_child_patterns_4() {
//        let mut graph = Hypergraph::default();
//        if let [a, b, _w, x, y, z] = graph.insert_tokens(
//            [
//                Token::Element('a'),
//                Token::Element('b'),
//                Token::Element('w'),
//                Token::Element('x'),
//                Token::Element('y'),
//                Token::Element('z'),
//            ])[..] {
//            // wxabyzabbyxabyz
//            let ab = graph.insert_pattern([a, b]);
//            let by = graph.insert_pattern([b, y]);
//            let yz = graph.insert_pattern([y, z]);
//            let xab = graph.insert_pattern([x, ab]);
//            let xaby = graph.insert_patterns([
//                vec![xab, y],
//                vec![x, a, by]
//            ]);
//            let xabyz = graph.insert_patterns([
//                vec![xaby, z],
//                vec![xab, yz]
//            ]);
//            // split xabyz at 2
//            let xa_graph = ChildStrings::from_node(
//                "xa",
//                vec![
//                    vec!["x", "a"],
//                ]
//            );
//            let byz_graph = ChildStrings::from_node(
//                "byz",
//                vec![
//                    vec!["by", "z"],
//                    vec!["b", "yz"],
//                ]
//            );
//
//            let (left, right) = graph.split_index_at_pos(xabyz, NonZeroUsize::new(2).unwrap());
//            let left = graph.pattern_child_strings(left);
//            let right = graph.pattern_child_strings(right);
//            assert_eq!(left, xa_graph, "left");
//            assert_eq!(right, byz_graph, "right");
//        } else {
//            panic!();
//        }
//    }
//    #[test]
//    fn split_child_patterns_5() {
//        let mut graph = Hypergraph::default();
//        if let [a, b, w, x, y, z] = graph.insert_tokens(
//            [
//                Token::Element('a'),
//                Token::Element('b'),
//                Token::Element('w'),
//                Token::Element('x'),
//                Token::Element('y'),
//                Token::Element('z'),
//            ])[..] {
//            // wxabyzabbyxabyz
//            let ab = graph.insert_pattern([a, b]);
//            let by = graph.insert_pattern([b, y]);
//            let yz = graph.insert_pattern([y, z]);
//            let xa = graph.insert_pattern([x, a]);
//            let xab = graph.insert_patterns([
//                vec![x, ab],
//                vec![xa, b],
//            ]);
//            let xaby = graph.insert_patterns([
//                vec![xab, y],
//                vec![xa, by]
//            ]);
//            let xabyz = graph.insert_patterns([
//                vec![xaby, z],
//                vec![xab, yz]
//            ]);
//            let wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);
//
//            // split wxabyzabbyxabyz at 3
//            let wxa_graph = ChildStrings::from_node(
//                "wxa",
//                vec![
//                    vec!["w", "xa"],
//                ]
//            );
//            let byzabbyxabyz_graph = ChildStrings::from_node(
//                "byzabbyxabyz",
//                vec![
//                    ["byz", "abbyxabyz"],
//                ]
//            );
//            let (left, right) = graph.split_index_at_pos(wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());
//            let left = graph.pattern_child_strings(left);
//            let right = graph.pattern_child_strings(right);
//            assert_eq!(left, wxa_graph, "left");
//            assert_eq!(right, byzabbyxabyz_graph, "right");
//        } else {
//            panic!();
//        }
//    }
//    fn split_child_patterns_6_impl() {
//        let mut graph = Hypergraph::default();
//        if let [a, b, w, x, y, z] = graph.insert_tokens(
//            [
//                Token::Element('a'),
//                Token::Element('b'),
//                Token::Element('w'),
//                Token::Element('x'),
//                Token::Element('y'),
//                Token::Element('z'),
//            ])[..] {
//            // wxabyzabbyxabyz
//            let ab = graph.insert_pattern([a, b]);
//            let by = graph.insert_pattern([b, y]);
//            let yz = graph.insert_pattern([y, z]);
//            let wx = graph.insert_pattern([w, x]);
//            let xab = graph.insert_pattern([x, ab]);
//            let xaby = graph.insert_patterns([
//                vec![xab, y],
//                vec![x, a, by]
//            ]);
//            let wxab = graph.insert_patterns([
//                vec![wx, ab],
//                vec![w, xab]
//            ]);
//            let wxaby = graph.insert_patterns([
//                vec![w, xaby],
//                vec![wx, a, by],
//                vec![wxab, y]
//            ]);
//            let xabyz = graph.insert_patterns([
//                vec![xaby, z],
//                vec![xab, yz]
//            ]);
//            let wxabyz = graph.insert_patterns([
//                vec![w, xabyz],
//                vec![wxaby, z],
//                vec![wx, ab, yz]
//            ]);
//            let wxa_graph = ChildStrings::from_node(
//                "wxa",
//                vec![
//                    vec!["wx", "a"],
//                ]
//            );
//            let byz_graph = ChildStrings::from_node(
//                "byz",
//                vec![
//                    vec!["by", "z"],
//                    vec!["b", "yz"],
//                ]
//            );
//
//            let (left, right) = graph.split_index_at_pos(wxabyz, NonZeroUsize::new(3).unwrap());
//            let left = graph.pattern_child_strings(left);
//            let right = graph.pattern_child_strings(right);
//            assert_eq!(left, wxa_graph, "left");
//            assert_eq!(right, byz_graph, "right");
//        } else {
//            panic!();
//        }
//    }
//    #[test]
//    fn split_child_patterns_6() {
//        split_child_patterns_6_impl()
//    }
//    //#[bench]
//    //fn bench_split_child_patterns_6(_bencher: &mut test::Bencher) {
//    //    split_child_patterns_6_impl()
//    //}
//    //#[bench]
//    //fn bench_split_child_patterns_3(b: &mut test::Bencher) {
//    //    b.iter(|| split_child_patterns_3_impl())
//    //}
//}
