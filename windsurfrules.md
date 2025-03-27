# Windsurfrules for rig-onchain-kit

## General Guidelines

1. **Only make changes relevant to our implementation**
   - Do not modify existing functionality unless necessary
   - Ensure backward compatibility with existing code
   - Focus on incremental improvements rather than complete rewrites

2. **Always choose the simplest implementation**
   - Favor readability over clever optimizations
   - Use existing patterns and libraries when available
   - Minimize dependencies and complexity

3. **Follow existing patterns and standards**
   - Maintain consistent code style with the rest of the codebase
   - Use the `#[tool]` macro for new tools following existing examples
   - Follow Rust best practices for error handling and async code

## Implementation-Specific Rules

### Cross-Chain Operations

1. **Tool Descriptions**
   - Include comprehensive descriptions for all tools
   - Document parameter types and expected formats
   - List supported chains and any limitations

2. **Error Handling**
   - Use `anyhow::Result` for error propagation
   - Provide clear error messages with context
   - Limit error message length (see existing pattern of taking first 300 chars)

3. **Chain and Token Identification**
   - Support multiple input formats (symbol, address, public key)
   - Validate inputs before making external API calls
   - Handle chain IDs consistently (support both numeric IDs and string identifiers)

4. **Transaction Safety**
   - Use `wrap_unsafe` for operations that interact with blockchain
   - Implement proper validation before executing transactions
   - Provide quote functionality before executing expensive operations

### Code Structure

1. **Module Organization**
   - Keep related functionality in the same module
   - Follow the existing directory structure
   - Use clear and descriptive names for functions and modules

2. **Testing**
   - Add tests for new functionality
   - Mock external dependencies in tests
   - Test both success and failure cases

3. **Documentation**
   - Document public APIs thoroughly
   - Include examples where appropriate
   - Update documentation when changing existing functionality

## Development Workflow

1. **Feature Implementation**
   - Start with a clear specification
   - Implement the simplest version that works
   - Refine and optimize only if necessary

2. **Code Review**
   - Ensure changes follow these rules
   - Verify backward compatibility
   - Check for potential security issues

3. **Integration**
   - Test changes with existing tools and agents
   - Verify cross-chain functionality works as expected
   - Ensure proper error handling and user feedback
