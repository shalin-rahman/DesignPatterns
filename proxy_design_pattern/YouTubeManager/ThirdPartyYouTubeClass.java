import java.util.HashMap;

/**
 * ThirdPartyYouTubeClass - Real Subject.
 *
 * Role in Proxy Pattern:
 *   - The actual service that does the real, heavy work.
 *   - Simulates slow network operations, HTTP handshakes, and remote downloads.
 *   - Direct usage causes redundant network calls and latency bottlenecks.
 */
public class ThirdPartyYouTubeClass implements ThirdPartyYouTubeLib {

    @Override
    public HashMap<String, Video> popularVideos() {
        connectToServer("http://www.youtube.com");
        return getRandomVideos();
    }

    @Override
    public Video getVideo(String videoId) {
        connectToServer("http://www.youtube.com/" + videoId);
        return getSomeVideo(videoId);
    }

    // ------------------------------------------------------------------------
    // Helper simulation methods mimicking real network latency
    // ------------------------------------------------------------------------

    private int experienceNetworkLatency() {
        int randomLatency = 100; // Simulated network delay in ms
        try {
            Thread.sleep(randomLatency);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        return randomLatency;
    }

    private void connectToServer(String server) {
        System.out.print("Connecting to " + server + "... ");
        experienceNetworkLatency();
        System.out.println("Connected!");
    }

    private HashMap<String, Video> getRandomVideos() {
        System.out.print("Downloading popular video metadata... ");
        experienceNetworkLatency();

        HashMap<String, Video> hmap = new HashMap<>();
        hmap.put("catzzzzzzzzz", new Video("sadgahasgda", "Catzzzz.avi"));
        hmap.put("mkafksda3jfn", new Video("mkafksda3jfn", "Dog play with ball.mp4"));
        hmap.put("dancesvideoo", new Video("asdfas3ffasd", "Dancing video.mp4"));
        hmap.put("dql4asda5ggs", new Video("3sdfgsd1j333", "Barcelona vs Real Madrid.mov"));
        hmap.put("mkuiggsdg333", new Video("54aw shock1", "Programmer meme song.mp4"));

        System.out.println("Done!");
        return hmap;
    }

    private Video getSomeVideo(String videoId) {
        System.out.print("Downloading video '" + videoId + "'... ");
        experienceNetworkLatency();
        Video video = new Video(videoId, "Sample Video Title (" + videoId + ")");
        System.out.println("Done!");
        return video;
    }
}
