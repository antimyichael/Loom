declare module 'tree-sitter-c-sharp';
declare module 'tree-sitter-java';

// The `ignore` package ships a CJS `module.exports = factory` binding.
// Node 22 ESM interop exposes this as the default export, so
// `import ignore from 'ignore'` works at runtime in both tsx and compiled dist/.
// We declare it with `export default` here to match that ESM import form.
declare module 'ignore' {
  interface Ignore {
    add(patterns: string | Ignore | readonly (string | Ignore)[]): this;
    filter(pathnames: readonly string[]): string[];
    createFilter(): (pathname: string) => boolean;
    ignores(pathname: string): boolean;
  }
  interface Options {
    ignorecase?: boolean;
    ignoreCase?: boolean;
    allowRelativePaths?: boolean;
  }
  function ignore(options?: Options): Ignore;
  namespace ignore {
    export function isPathValid(pathname: string): boolean;
    export { Ignore, Options };
  }
  export default ignore;
}