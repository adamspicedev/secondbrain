export default {
  branches: ["main"],
  tagFormat: "v${version}",
  plugins: [
    [
      "@semantic-release/commit-analyzer",
      {
        preset: "conventionalcommits",
        releaseRules: [
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
        preset: "conventionalcommits"
      }
    ],
    "@semantic-release/github"
  ]
};
