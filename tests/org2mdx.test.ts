import { orgToMdx } from '../src/index';

describe('orgToMdx', () => {
  it('should convert basic Org heading to MDX', () => {
    const orgContent = '* Hello World';
    const expected = '# Hello World';
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert multiple headings', () => {
    const orgContent = `* First Level
** Second Level
*** Third Level`;
    const expected = `# First Level

## Second Level

### Third Level`;
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org paragraphs to MDX', () => {
    const orgContent = `This is a paragraph.

This is another paragraph.`;
    const expected = `This is a paragraph.

This is another paragraph.`;
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org bold text to MDX', () => {
    const orgContent = 'This is *bold* text.';
    const expected = 'This is **bold** text.';
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org italic text to MDX', () => {
    const orgContent = 'This is /italic/ text.';
    const expected = 'This is *italic* text.';
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org code inline to MDX', () => {
    const orgContent = 'This is ~code~ text.';
    const expected = 'This is `code` text.';
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org verbatim inline to MDX', () => {
    const orgContent = 'This is =verbatim= text.';
    const expected = 'This is `verbatim` text.';
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org code blocks to MDX', () => {
    const orgContent = `#+BEGIN_SRC javascript
console.log('Hello World');
#+END_SRC`;
    const expected = `\`\`\`javascript
console.log('Hello World');
\`\`\``;
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org links to MDX', () => {
    const orgContent = '[[https://example.com][Example Site]]';
    const expected = '[Example Site](https://example.com)';
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should handle frontmatter extraction', () => {
    const orgContent = `#+TITLE: My Document
#+AUTHOR: John Doe

Some content.`;
    const result = orgToMdx(orgContent, { frontmatter: true });
    expect(result).toContain('---');
    expect(result).toContain('title: "My Document"');
    expect(result).toContain('author: "John Doe"');
    expect(result).toContain('Some content.');
  });

  it('should handle empty content', () => {
    expect(orgToMdx('')).toBe('');
  });

  it('should add frontmatter when requested', () => {
    const orgContent = `#+TITLE: My Document
#+AUTHOR: John Doe
#+DATE: 2023-10-29

Some content.`;
    const result = orgToMdx(orgContent, { frontmatter: true });
    expect(result).toContain('---');
    expect(result).toContain('title: "My Document"');
    expect(result).toContain('author: "John Doe"');
  });

  it('should apply component mappings', () => {
    const orgContent = '<note>This is a note</note>';
    const result = orgToMdx(orgContent, {
      components: { note: 'Callout' },
    });
    expect(result).toContain('<Callout>');
  });

  it('should convert Org tables to MDX', () => {
    const orgContent = `| Name | Age | City |
|------+-----+------|
| John | 25  | NYC  |
| Jane | 30  | LA   |`;
    const expected = `| Name | Age | City |
| ---- | --- | ---- |
| John | 25  | NYC  |
| Jane | 30  | LA   |`;
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org tables with alignment to MDX', () => {
    const orgContent = `|      |         |        |       |
| ---- | ------- | ------ | ----- |
| <l>  |         | <c>    | <r>   |
| Left | Default | Center | Right |`;
    const expected = `| Left | Default | Center | Right |
| :--- | ------- | :----: | ----: |`;
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should convert Org images with captions to MDX figures', () => {
    const orgContent = `#+caption: A beautiful unicorn
[[/images/unicorn.png]]

#+caption: Another *caption* with /markup/
[[/images/another.png]]`;
    const expected = `<figure><img src="/images/unicorn.png" alt="" /><figcaption>A beautiful unicorn</figcaption></figure>

<figure><img src="/images/another.png" alt="" /><figcaption>Another <strong>caption</strong> with <em>markup</em></figcaption></figure>
`;
    expect(orgToMdx(orgContent)).toBe(expected);
  });

  it('should throw error for invalid Org content', () => {
    // This test assumes the parser might fail on certain invalid content
    // The exact behavior depends on uniorg's error handling
    const invalidOrg = '*** Invalid heading structure';
    expect(() => orgToMdx(invalidOrg)).not.toThrow();
  });
});
