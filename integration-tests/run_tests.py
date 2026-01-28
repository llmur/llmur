#!/usr/bin/env python3
"""
Test runner script for the API integration tests.
Usage: python run_tests.py [options]
"""

import sys
import subprocess
import argparse
import os

# Available test suites
TEST_SUITES = {
    'all': [],  # Empty list means run all tests
    'users': ['test_users.py'],
    'projects': ['test_projects.py'],
    'memberships': ['test_memberships.py'],
    'connections': ['test_connections.py'],
    'crud': ['test_users.py', 'test_projects.py', 'test_memberships.py', 'test_connections.py'],  # All CRUD tests
}


def run_tests(suite=None, test_file=None, pattern=None, verbose=False, html_report=False):
    """Run the API integration tests"""

    cmd = [sys.executable, '-m', 'pytest']

    # Determine what to run
    if test_file:
        # Specific file takes precedence
        cmd.append(test_file)
    elif suite and suite in TEST_SUITES:
        # Run predefined test suite
        if TEST_SUITES[suite]:  # If not empty (i.e., not 'all')
            cmd.extend(TEST_SUITES[suite])
    elif pattern:
        # Run tests matching pattern
        cmd.extend(['-k', pattern])

    if verbose:
        cmd.append('-v')

    if html_report:
        # Create reports directory if it doesn't exist
        os.makedirs('reports', exist_ok=True)
        cmd.extend(['--html=reports/test_report.html', '--self-contained-html'])

    # Add useful pytest options
    cmd.extend([
        '--color=yes',  # Colored output
        '-l',  # Show local variables in tracebacks
        '--tb=short',  # Shorter traceback format
        '--strict-markers',  # Strict marker checking
    ])

    print(f"Running command: {' '.join(cmd)}")
    print(f"Available test files: {[f for f in os.listdir('.') if f.startswith('test_') and f.endswith('.py')]}")

    result = subprocess.run(cmd)

    return result.returncode


def list_available_tests():
    """List all available test suites and files"""
    print("Available test suites:")
    for suite, files in TEST_SUITES.items():
        file_list = files if files else "all test files"
        print(f"  {suite}: {file_list}")

    print("\nAvailable test files:")
    test_files = [f for f in os.listdir('.') if f.startswith('test_') and f.endswith('.py')]
    for test_file in test_files:
        print(f"  {test_file}")


def main():
    parser = argparse.ArgumentParser(
        description='Run API integration tests',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
Examples:
  python run_tests.py                    # Run all tests
  python run_tests.py --suite users      # Run only user tests
  python run_tests.py --file test_users.py # Run specific file
  python run_tests.py --pattern "create" # Run tests with "create" in name
  python run_tests.py --list             # Show available options
        '''
    )

    parser.add_argument('--suite', '-s', choices=TEST_SUITES.keys(),
                        help='Test suite to run')
    parser.add_argument('--file', '-f', help='Specific test file to run')
    parser.add_argument('--pattern', '-k', help='Run tests matching pattern')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--html', action='store_true', help='Generate HTML report')
    parser.add_argument('--list', '-l', action='store_true', help='List available tests')

    args = parser.parse_args()

    if args.list:
        list_available_tests()
        return

    exit_code = run_tests(
        suite=args.suite,
        test_file=args.file,
        pattern=args.pattern,
        verbose=args.verbose,
        html_report=args.html
    )

    sys.exit(exit_code)


if __name__ == '__main__':
    main()
