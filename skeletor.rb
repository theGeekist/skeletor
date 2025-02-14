class Skeletor < Formula
  desc "A super optimised Rust scaffolding tool with snapshot annotations"
  homepage "https://github.com/jasonnathan/skeletor"
  url "https://github.com/jasonnathan/skeletor/releases/download/v2.2.17/skeletor-macos-x86_64-apple-darwin.tar.gz" # link to your tar.gz binary
  sha256 "d848937300fecbdc6f0463ab367414e9c976786be15358b973751c0ec7a90c53"
  license "MIT"

  def install
    bin.install "skeletor"
  end

  test do
    system "#{bin}/skeletor", "--version"
  end
end
