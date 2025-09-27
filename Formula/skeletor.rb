class Skeletor < Formula
  desc "Two-way scaffolding CLI (apply + snapshot)"
  homepage "https://github.com/theGeekist/skeletor"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/theGeekist/skeletor/releases/download/v0.3.8/skeletor-macos-aarch64-apple-darwin.tar.gz"
      sha256 ""
    end
    on_intel do
      url "https://github.com/theGeekist/skeletor/releases/download/v0.3.8/skeletor-macos-x86_64-apple-darwin.tar.gz"
      sha256 ""
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/theGeekist/skeletor/releases/download/v0.3.8/skeletor-ubuntu-x86_64-unknown-linux-gnu.tar.gz"
      sha256 ""
    end
  end

  def install
    bin.install "skeletor"
  end

  test do
    assert_match "skeletor", shell_output("#{bin}/skeletor --help")
  end

  livecheck do
    url :stable
    strategy :github_latest
  end
end
