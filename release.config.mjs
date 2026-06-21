export default {
  branches: ["release"],
  tagFormat: "v${version}",
  parserOpts: {
    headerPattern: /^(\w+)(?:\(([^)]+)\))?!?:?\s*(.+)$/,
    headerCorrespondence: ["type", "scope", "subject"]
  },
  plugins: [
    [
      "@semantic-release/commit-analyzer",
      {
        preset: "conventionalcommits",
        parserOpts: {
          headerPattern: /^(\w+)(?:\(([^)]+)\))?!?:?\s*(.+)$/,
          headerCorrespondence: ["type", "scope", "subject"]
        },
        releaseRules: [
          { type: "feat", release: "minor" },
          { type: "fix", release: "patch" },
          { subject: "Feat/*", release: "minor" },
          { subject: "feat/*", release: "minor" },
          { subject: "Add *", release: "minor" },
          { subject: "add *", release: "minor" },
          { subject: "Fix *", release: "patch" },
          { subject: "fix *", release: "patch" },
          { subject: "Refactor *", release: "patch" },
          { subject: "refactor *", release: "patch" },
          { subject: "Update *", release: "patch" },
          { subject: "update *", release: "patch" }
        ]
      }
    ],
    [
      "@semantic-release/release-notes-generator",
      {
        preset: "conventionalcommits",
        parserOpts: {
          headerPattern: /^(\w+)(?:\(([^)]+)\))?!?:?\s*(.+)$/,
          headerCorrespondence: ["type", "scope", "subject"]
        },
        presetConfig: {
          types: [
            { type: "feat", section: "Features", hidden: false },
            { type: "Feat", section: "Features", hidden: false },
            { type: "add", section: "Features", hidden: false },
            { type: "Add", section: "Features", hidden: false },
            { type: "fix", section: "Bug Fixes", hidden: false },
            { type: "Fix", section: "Bug Fixes", hidden: false },
            { type: "refactor", section: "Refactors", hidden: false },
            { type: "Refactor", section: "Refactors", hidden: false },
            { type: "update", section: "Updates", hidden: false },
            { type: "Update", section: "Updates", hidden: false }
          ]
        }
      }
    ],
    [
      "@semantic-release/github",
      {
        assets: [
          {
            path: "src-tauri/target/release/bundle/dmg/*.dmg",
            label: "Second Brain macOS DMG"
          }
        ]
      }
    ]
  ]
};
