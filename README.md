### booktyping has been renamed and moved to JesseCSlater/scrivenwright. This repository will no longer be used

## booktyping

![usage-example-gif](https://github.com/JesseCSlater/booktyping/blob/master/usage-example.gif)

booktyping is a simple commandline tool for practicing typing accuracy while reading a book.

# installation

booktyping has only been tested on linux, but should work on Windows and MacOS. The only dependency is rust. 

Clone this repository and run
```bash
cargo build --release
```
This will generate an executable file in booktyping/target/release/.

# usage
Find a text copy of your favorite book, and place it in $HOME/.booktyping/{book_title}.txt.
Now run booktyping with 
```bash
./booktyping {book_title}
```

Your progress will be automatically saved, and JSON logs of your keypresses and your sample completions will be saved in $HOME/.booktyping/{book_title}/keypresses.json and $HOME/.booktyping/{book_title}/tests.json respectively.
