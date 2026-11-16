import java.util.HashMap;

/**
 * YouTubeCacheProxy - Caching Proxy.
 *
 * Role in Proxy Pattern:
 *   - Implements the same interface (ThirdPartyYouTubeLib) as the real service.
 *   - Wraps the RealSubject (ThirdPartyYouTubeClass) via composition.
 *   - Intercepts requests to check local memory cache before hitting slow network.
 *   - Performs lazy loading and transparent caching without altering client or service code.
 */
public class YouTubeCacheProxy implements ThirdPartyYouTubeLib {
    private final ThirdPartyYouTubeLib youtubeService;
    private HashMap<String, Video> cachePopular = new HashMap<>();
    private final HashMap<String, Video> cacheAll = new HashMap<>();

    public YouTubeCacheProxy() {
        this.youtubeService = new ThirdPartyYouTubeClass();
    }

    public YouTubeCacheProxy(ThirdPartyYouTubeLib youtubeService) {
        this.youtubeService = youtubeService;
    }

    @Override
    public HashMap<String, Video> popularVideos() {
        if (cachePopular.isEmpty()) {
            cachePopular = youtubeService.popularVideos();
        } else {
            System.out.println("Retrieved list from cache.");
        }
        return cachePopular;
    }

    @Override
    public Video getVideo(String videoId) {
        Video video = cacheAll.get(videoId);
        if (video == null) {
            video = youtubeService.getVideo(videoId);
            cacheAll.put(videoId, video);
        } else {
            System.out.println("Retrieved video '" + videoId + "' from cache.");
        }
        return video;
    }

    /**
     * Resets/invalidates all cached data.
     */
    public void reset() {
        cachePopular.clear();
        cacheAll.clear();
    }
}
