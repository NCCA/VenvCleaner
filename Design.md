# Venv cleaner

## cli mode

Is an 3 mode application to help manage and clean up .venv folders on mac and linux.

The fist mode is cli, this allows the user to run the tool venv_cleaner [dir] with the following command lines

-r recurse and search from the current directory
-f force always delete

The tool will search for a .venv in the current folder or one passed and prompt the user if found if it should be deleted. If yes then the .venv folder will be removed. If the -r or -f flags are used

-q query

will recurse the directory passed or current directory and print out the path / and size of the .venv folder. The size should be reported in either Mb or Gb depending upon the size.

## tui

This will use the rust tui library to give a console based application. this is activated using the --tui flag.

It will open the current folder and present a view of each .venv folders found by recursing.

It will display in a list the following information

location  size in bytes last used data created date

There should be the ability to open a new folder , delete the selected .venv in the list as well as order by date of creation

## Gui

This will as the same functionality as the tui version but use a Qt6 based GUI rather than a tui.

