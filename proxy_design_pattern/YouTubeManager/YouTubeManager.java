import java.util.HashMap;

/**
 * YouTubeManager - Client / Consumer Component.
 *
 * Role in Proxy Pattern:
 *   - Interacts exclusively with the ThirdPartyYouTubeLib interface.
 *   - Completely decoupled from whether it talks directly to the RealSubject
 *     or through the Caching Proxy.
 *   - Open for extension, closed for modification (Open/Closed Principle).
 */
public class YouTubeManager {
    protected ThirdPartyYouTubeLib service;

    public YouTubeManager(ThirdPartyYouTubeLib service) {
        this.service = service;
    }

    /**
     * Renders a video details playback page.
     *
     * @param id the video ID to display.
     */
    public void renderVideoPage(String id) {
        Video video = service.getVideo(id);
        System.out.println("\n-------------------------------");
        System.out.println("Video page (rendered via Manager):");
        System.out.println("ID: " + video.id);
        System.out.println("Title: " + video.title);
        System.out.println("Data: " + video.data);
        System.out.println("-------------------------------\n");
    }

    /**
     * Renders the list of popular video thumbnails and titles.
     */
    public void renderPopularVideos() {
        HashMap<String, Video> list = service.popularVideos();
        System.out.println("\n-------------------------------");
        System.out.println("Most popular videos on YouTube (rendered via Manager):");
        for (Video video : list.values()) {
            System.out.println("ID: " + video.id + " / Title: " + video.title);
        }
        System.out.println("-------------------------------\n");
    }
}
