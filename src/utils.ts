/**
 * utils.ts - Converts a relative file path to its Obsidian note filename
 * e.g. 'Scripts/Corgi.cs' -> 'Scripts__Corgi.cs'
 * e.g. 'src/types.ts' -> 'src__types.ts'
 */
export function filePathToNoteId(filePath: string): string {
  return filePath.replace(/\//g, '__').replace(/\\/g, '__');
}
