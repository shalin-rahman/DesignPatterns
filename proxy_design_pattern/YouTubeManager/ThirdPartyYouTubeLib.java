import java.util.HashMap;

/**
 * ThirdPartyYouTubeLib - Service Interface (Subject).
 *
 * Role in Proxy Pattern:
 *   - Declares the common interface for both the RealSubject (ThirdPartyYouTubeClass)
 *     and the Proxy (YouTubeCacheProxy).
 *   - Allows the client (YouTubeManager) to treat real service and proxy interchangeably.
 */
public interface ThirdPartyYouTubeLib {

    /**
     * Retrieves a map of popular videos.
     *
     * @return map of video ID to Video object.
     */
    HashMap<String, Video> popularVideos();

    /**
     * Downloads/fetches a specific video by its unique ID.
     *
     * @param videoId unique identifier of the video.
     * @return Video object.
     */
    Video getVideo(String videoId);
}
