# Ramify

Ramify is a Rust library for generating *branch diagrams* to visualize hierarchical data.
```txt
0       0         0    
├╮      ├┬╮       ├┬╮  
1├╮     │1├╮      │1│  
│2│     ││2│      2│╰─╮
│3│     │3││      │╰─╮│
├╮│     │╭╯│      ├┬╮││
4││     ││╭┼╮     │3│││
 5│     │││4│     4╭╯││
╭╯6     ││5╭╯      5╭╯│
7       │6╭╯        6╭╯
        7╭╯          7
         8             
```
See the [gallery](#gallery) for more images.

This library is specifically designed for ordered data: this is closer to the output of
`git log --graph --all` than the output of `tree`.
A prototypical application is to visualize the undo-tree of a text file.
The order is the timestamp of the edit, and the tree structure results from the undo relation.

## Key features

- Single-pass layout algorithm which optimizes for width and appearance.
- Memory efficient streaming implementation: new vertices are not requested until the
  parent vertex has been rendered.
- Robust support for metadata via annotations and custom marker characters.
- Generic over ordered hierarchical data with efficient iteration over immediate children.
- No dependencies other than the standard library.

Read the [API documentation](docs.rs/crates/ramify) for more detail!

## Gallery

