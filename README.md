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
See the [gallery](https://github.com/alexrutar/ramify#gallery) for more images.

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

Interested?
Check out the [API documentation](https://docs.rs/ramify/latest/ramify/) for more detail!

Basic examples can be found in the [examples folder](https://github.com/alexrutar/ramify/tree/master/examples).

## Gallery
Basic examples with no annotation and various node markers
```txt
 0        0             0         ◉
 ├┬╮      ├┬╮           ├┬╮       ├┬╮
 │1├╮     │1├┬╮         │1│       │✕│
 ││2│     │││2├┬╮       2│╰╮      │╭┼╮
 │3││     │││││3├╮      │╰╮│      ││◉├┬╮
 │╭╯│     │││││││4      ├╮││      ││││◉│
 ││╭┼╮    ││││5│││      3│││      │◉││││
 │││4│    ││6│╭╯││      ╭╯││      │ ✕│││
 ││5╭╯    ││ 7│╭╯│      │╭┤│      │╭─╯◉│
 │6╭╯     ││╭─╯│ 8      │4││      │◉ ╭╯│
 7╭╯      │││  9╭╯      5╭╯│      ├╮╭╯╭╯
  8       │││╭┬╯│        6╭┤      ✕││╭╯
          ││││a╭╯         7│      ╭╯✕│
          ││b│╭╯           8      ◉╭─╯
          │c╭╯│                   │✕
          │││ d                   ◉
          ││e
          │f
          g
```
The first example above, with annotations associated with some vertices.
```txt
0
├┬╮
│1├╮ An annotation
││││ with two lines
││2│
│3││  Another annotation
│╭╯│
││╭┼╮
│││4│ An annotation
│││╭╯ split over
││││  three lines
││5│
│6╭╯
7╭╯
 8
```
The same example, but with extra padding after the annotation and with 'width slack' to decrease the height at the cost of making the tree wider.
```txt
0
├┬╮
│1│   An annotation
││├╮  with two lines
││││
││2│
││││
│3│╰╮  Another annotation
│╭╯╭┼╮
││╭╯4│ An annotation
│││╭─╯ split over
││││   three lines
││││
││5│
││╭╯
│6│
│╭╯
7│
╭╯
8
```
