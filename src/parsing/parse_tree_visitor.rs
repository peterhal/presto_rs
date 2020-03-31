use super::ParseTree;

/// Visit all children of a node in a parse tree calling pre_visit before
/// any children of a node have been visited and calling post_visit after
/// all children of a node have been visited.
pub fn visit_children<'a, F1, F2>(tree: &'a ParseTree<'a>, pre_visit: &mut F1, post_visit: &mut F2)
where
    F1: FnMut(&'a ParseTree<'a>) -> (),
    F2: FnMut(&'a ParseTree<'a>) -> (),
{
    for child in tree.children() {
        visit(child, pre_visit, post_visit);
    }
}

/// Visit all nodes in a parse tree calling pre_visit before
/// any children of a node have been visited and calling post_visit after
/// all children of a node have been visited.
pub fn visit<'a, F1, F2>(tree: &'a ParseTree<'a>, pre_visit: &mut F1, post_visit: &mut F2)
where
    F1: FnMut(&'a ParseTree<'a>) -> (),
    F2: FnMut(&'a ParseTree<'a>) -> (),
{
    pre_visit(tree);
    visit_children(tree, pre_visit, post_visit);
    post_visit(tree);
}

/// Visit all nodes in a parse tree calling post_visit after
/// all children of a node have been visited.
pub fn visit_post_order<'a, F>(tree: &'a ParseTree<'a>, post_visit: &mut F)
where
    F: FnMut(&'a ParseTree<'a>) -> (),
{
    visit(tree, &mut |_tree| (), post_visit);
}

/// Visit all nodes in a parse tree calling pre_visit before
/// any children of a node have been visited.
pub fn visit_pre_order<'a, F>(tree: &'a ParseTree<'a>, pre_visit: &mut F)
where
    F: FnMut(&'a ParseTree<'a>) -> (),
{
    visit(tree, pre_visit, &mut |_tree| ());
}
