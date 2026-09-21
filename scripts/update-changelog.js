// Usage: node scripts/update-changelog.js <version>
// Prepends a section for <version> to CHANGELOG.md, built from the commit subjects since the previous
// release tag. Commits should follow Conventional Commits (feat:, fix:, docs:, ...). Safe to re-run.
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const version = (process.argv[2] || '').replace(/^v/, '');
if (!version) {
  console.error('Usage: update-changelog.js <x.y.z>');
  process.exit(1);
}

const root = path.join(__dirname, '..');
const changelogPath = path.join(root, 'CHANGELOG.md');
const MARKER = '<!-- changelog:start -->';
const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();

let changelog = fs.readFileSync(changelogPath, 'utf8');
if (changelog.includes(`## [${version}]`)) {
  console.log(`CHANGELOG.md already has ${version}; nothing to do`);
  process.exit(0);
}
if (!changelog.includes(MARKER)) {
  console.error(`CHANGELOG.md is missing the ${MARKER} marker`);
  process.exit(1);
}

// Commits since the previous release tag (or all history for the first release)
let range = 'HEAD';
try {
  const prev = git('describe', '--tags', '--abbrev=0', '--match', 'v*', `v${version}^`);
  range = `${prev}..HEAD`;
} catch {
  try {
    const prev = git('describe', '--tags', '--abbrev=0', '--match', 'v*', 'HEAD');
    if (prev !== `v${version}`) range = `${prev}..HEAD`;
  } catch {
    // no earlier tags
  }
}

const SECTIONS = [
  ['Added', ['feat']],
  ['Fixed', ['fix']],
  ['Performance', ['perf']],
  ['Changed', ['refactor', 'style', 'revert']],
  ['Documentation', ['docs']],
  ['Maintenance', ['build', 'ci', 'chore', 'test']],
];
const typeToSection = new Map(SECTIONS.flatMap(([name, types]) => types.map((t) => [t, name])));
const grouped = new Map(SECTIONS.map(([name]) => [name, []]));
grouped.set('Other', []);
const breaking = [];

const log = git('log', range, '--no-merges', '--pretty=format:%s%x09%h');
for (const line of log ? log.split('\n') : []) {
  const [subject, hash] = line.split('\t');
  if (/^chore\(release\)/.test(subject)) continue;

  const m = subject.match(/^(\w+)(?:\(([^)]+)\))?(!)?:\s*(.+)$/);
  const type = m ? m[1].toLowerCase() : null;
  const scope = m && m[2] ? `**${m[2]}:** ` : '';
  const text = m ? m[4] : subject;
  const entry = `- ${scope}${text} (${hash})`;

  if (m && m[3]) {
    breaking.push(entry);
  } else {
    grouped.get(typeToSection.get(type) || 'Other').push(entry);
  }
}

const date = new Date().toISOString().slice(0, 10);
let section = `## [${version}] - ${date}\n`;
if (breaking.length) section += `\n### ⚠ Breaking changes\n\n${breaking.join('\n')}\n`;
let any = breaking.length > 0;
for (const [name, entries] of grouped) {
  if (!entries.length) continue;
  any = true;
  section += `\n### ${name}\n\n${entries.join('\n')}\n`;
}
if (!any) section += '\n- No notable changes.\n';

changelog = changelog.replace(MARKER, `${MARKER}\n\n${section.trimEnd()}`);
fs.writeFileSync(changelogPath, changelog);
console.log(`CHANGELOG.md updated for ${version} (${range})`);
