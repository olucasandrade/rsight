class Rsight < Formula
  desc "Fast TUI search for files, content, and AI conversations"
  homepage "https://github.com/lucasandrade/rsight"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/lucasandrade/rsight/releases/download/v#{version}/rsight-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "<sha256-arm64>"
    else
      url "https://github.com/lucasandrade/rsight/releases/download/v#{version}/rsight-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "<sha256-x86_64>"
    end
  end

  def install
    bin.install "rsight"
  end

  test do
    assert_predicate bin/"rsight", :exist?
  end
end
