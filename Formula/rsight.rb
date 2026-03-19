class Rsight < Formula
  desc "Fast TUI search for files, content, and AI conversations"
  homepage "https://github.com/olucasandrade/rsight"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/olucasandrade/rsight/releases/download/v#{version}/rsight-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "2fd4c6c6fda6be529f24f361219ea3d2f60cc1852b18a6bd8f44fc015cbe7174" # arm64
    else
      url "https://github.com/olucasandrade/rsight/releases/download/v#{version}/rsight-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "46f983ff61177dc5431cbbfda095c262723098855ed5226aaad28a8747951a9a" # x86_64
    end
  end

  def install
    bin.install "rsight"
  end

  test do
    assert_predicate bin/"rsight", :exist?
  end
end
