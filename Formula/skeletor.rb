class Skeletor < Formula
  desc "Two-way scaffolding CLI (apply + snapshot)"
  homepage "https://github.com/theGeekist/skeletor"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/theGeekist/skeletor/releases/download/v0.3.7/skeletor-macos-aarch64-apple-darwin.tar.gz"
      sha256 "8b9eb5fa70813c949d0acbf63880137f451ddef82ebe9c988549e0f44b1ef586"
    end
    on_intel do
      url "https://github.com/theGeekist/skeletor/releases/download/v0.3.7/skeletor-macos-x86_64-apple-darwin.tar.gz"
      sha256 "15302226f5796fa032a0c141717578532cfd160dcf36ae2d0721986a3be423bd"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/theGeekist/skeletor/releases/download/v0.3.7/skeletor-ubuntu-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "fbf247fedcb1fbdfc641bd5f685673eaef755d2daed657c1cba0079b6ca9efdd"
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
