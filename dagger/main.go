package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"dagger.io/dagger"
)

func main() {
	ctx := context.Background()

	client, err := dagger.Connect(ctx, dagger.WithLogOutput(os.Stderr))
	if err != nil {
		panic(err)
	}
	defer client.Close()

	if len(os.Args) < 2 {
		fmt.Println("Usage: go run main.go <command>")
		fmt.Println("Commands: test-local, release")
		os.Exit(1)
	}

	command := os.Args[1]

	switch command {
	case "test-local":
		if err := testLocal(ctx, client); err != nil {
			fmt.Fprintf(os.Stderr, "Error running test-local: %v\n", err)
			os.Exit(1)
		}
	case "release":
		if err := release(ctx, client); err != nil {
			fmt.Fprintf(os.Stderr, "Error running release: %v\n", err)
			os.Exit(1)
		}
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", command)
		os.Exit(1)
	}
}

// getProjectSourceDirectory returns a filtered directory containing only essential project files
func getProjectSourceDirectory(client *dagger.Client) (*dagger.Directory, error) {
	wd, err := os.Getwd()
	if err != nil {
		return nil, fmt.Errorf("failed to get working directory: %w", err)
	}
	projectRoot := filepath.Dir(wd)

	// Create filtered directory excluding large/unnecessary files
	sourceDir := client.Host().Directory(projectRoot, dagger.HostDirectoryOpts{
		Exclude: []string{
			"target",
			"node_modules",
			".git",
			"dagger",
			".vscode",
			".idea",
			"dist",
			"build",
			".DS_Store",
			"*.log",
			"*.tmp",
			"*.bak",
		},
	})

	return sourceDir, nil
}

// testLocal mirrors the CI pipeline locally
func testLocal(ctx context.Context, client *dagger.Client) error {
	fmt.Println("🧪 Running Zephyrite test suite locally (mirroring CI)")

	sourceDir, err := getProjectSourceDirectory(client)
	if err != nil {
		return fmt.Errorf("failed to get project source directory: %w", err)
	}

	container := client.Container().
		From("rust:1.85").
		WithWorkdir("/workspace").
		WithDirectory("/workspace", sourceDir).
		WithExec([]string{"apt-get", "update"}).
		WithExec([]string{"apt-get", "install", "-y", "curl"}).
		WithExec([]string{"rustup", "component", "add", "rustfmt", "clippy"})

	container = container.WithExec([]string{"cargo", "install", "cargo-nextest", "--locked"})

	fmt.Println("📋 Checking formatting...")
	container = container.WithExec([]string{"cargo", "fmt", "--all", "--", "--check"})

	fmt.Println("🔍 Running clippy...")
	container = container.WithExec([]string{"cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"})

	fmt.Println("🔨 Building...")
	container = container.WithExec([]string{"cargo", "build", "--verbose"})

	fmt.Println("🧪 Running tests with nextest...")
	container = container.WithExec([]string{"cargo", "nextest", "run", "--config-file", ".cargo/nextest.toml", "--profile", "ci"})

	fmt.Println("📚 Running doctests...")
	container = container.WithExec([]string{"cargo", "test", "--doc", "--verbose"})

	_, err = container.Stdout(ctx)
	if err != nil {
		return fmt.Errorf("test pipeline failed: %w", err)
	}

	fmt.Println("✅ All tests passed locally!")
	return nil
}

// release handles building release artifacts
func release(ctx context.Context, client *dagger.Client) error {
	fmt.Println("🚀 Building Zephyrite release artifacts")

	sourceDir, err := getProjectSourceDirectory(client)
	if err != nil {
		return fmt.Errorf("failed to get project source directory: %w", err)
	}

	container := client.Container().
		From("rust:1.85").
		WithWorkdir("/workspace").
		WithDirectory("/workspace", sourceDir).
		WithExec([]string{"rustup", "component", "add", "rustfmt", "clippy"})

	fmt.Println("🔨 Building release binary...")
	container = container.WithExec([]string{"cargo", "build", "--release"})

	binary := container.File("/workspace/target/release/zephyrite")

	_, err = binary.Export(ctx, "./target/release/zephyrite")
	if err != nil {
		return fmt.Errorf("failed to export binary: %w", err)
	}

	fmt.Println("✅ Release binary built successfully!")
	fmt.Println("📦 Binary available at: ./target/release/zephyrite")
	return nil
}
