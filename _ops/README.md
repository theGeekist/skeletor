# ⚠️ SAFE TESTING DIRECTORY

This directory is the **ONLY** safe place to test skeletor functionality.

## 🛡️ SAFETY RULES

### ✅ SAFE OPERATIONS
- Create YAML configs in `test-configs/`
- Run `skeletor apply` from within `_ops/` subdirectories
- Test `--overwrite` flag safely with disposable files
- Use `--dry-run` first, always

### ❌ FORBIDDEN OPERATIONS
- **NEVER** run skeletor commands from project root
- **NEVER** create YAML files that reference `../src/` or other project paths
- **NEVER** use `--overwrite` outside of this directory

## 📁 Directory Structure

```
_ops/
├── test-configs/    # Safe YAML configuration files
├── test-output/     # Output from skeletor operations
└── README.md        # This safety guide
```

## 🚨 EMERGENCY PROTOCOL

Before ANY skeletor apply operation:
1. Run `pwd` - verify you're in `_ops/` subdirectory
2. Use `--dry-run` first to preview operations
3. Never include project source paths in YAML configs
4. Only use `--overwrite` with disposable test files

**Remember: One wrong `skeletor apply --overwrite` in the project root could destroy the entire codebase!**