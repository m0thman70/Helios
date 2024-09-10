# Atto!

this is atto, a new code editor similar to nano,vim and emacs that came to existance with alot of AI and ALOT of Bullsh** but here we are, it uses crossterm and tui libraries


## Bugs I am aware of:

cursor highlighting instead of taking up one cell


### Deps:

  - rust
    
### Notes:

currently its in very early stages, this is version 0.0.2 so be sure to know that there is bugs 
bugs: cant write to an empty file the file needs to have stuff in it and you need a file to use atto

### Binds:

`CTRL + Q` -> exit

`CTRL + HJKL` -> move cursor

`CTRL + W` -> save file

`CTRL + R` -> reload file

### Next steps:

So after some people used it, here is a list of a few things that people seemingly want to see:

- add something similar to which-key
- syntax highlighting -> possibly tree sitter
- commands -> no idea how
- a few people adsked for modes but thats gonna be sidelined

### Install:

`git clone https://github.com/m0thman70/Atto`

`cd atto`

`cargo build --release`

`cd target/release/atto`

`chmod +x atto`

`./atto`

and boom you got atto compiled, if you wanna install it, give me some time

also atto does not have the capability to create files, please use `touch`  
