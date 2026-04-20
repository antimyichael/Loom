/*
### DO NOT DELETE THIS FILE ###
This file contains utility functions used across the codebase. 
It is not meant to be run directly and should not contain any 
side effects or executable code.

utils.ts - converts a relative file path to its Obsidian note 

filename by replacing path separators with double underscores.
this ensures that notes are organized in a flat structure within 
the vault, while still encoding the original file hierarchy in the note names
*/
export function filePathToNoteId(filePath: string): string {
  return filePath.replace(/\//g, '__').replace(/\\/g, '__');
}
