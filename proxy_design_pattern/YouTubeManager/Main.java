/**
 * Main - Benchmark Driver for the Proxy Design Pattern Demo.
 *
 * Demonstrates:
 *   1. Naive Execution: Direct calls to ThirdPartyYouTubeClass (every call incurs network delay).
 *   2. Cached Execution: Calls intercepted by YouTubeCacheProxy (cached results served instantly).
 *   3. Performance comparison and time saved.
 */
public class Main {

    public static void main(String[] args) {
        YouTubeManager naiveManager = new YouTubeManager(new ThirdPartyYouTubeClass());
        YouTubeManager smartManager = new YouTubeManager(new YouTubeCacheProxy());

        System.out.println("==================================================");
        System.out.println("Running Naive Manager (Direct Real Subject Calls):");
        System.out.println("==================================================");
        long naiveTime = test(naiveManager);

        System.out.println("\n==================================================");
        System.out.println("Running Smart Manager (Caching Proxy Interception):");
        System.out.println("==================================================");
        long smartTime = test(smartManager);

        System.out.println("\n==================================================");
        System.out.println("BENCHMARK SUMMARY:");
        System.out.println("Time taken with Real Subject (Naive): " + naiveTime + "ms");
        System.out.println("Time taken with Proxy (Cached):       " + smartTime + "ms");
        System.out.println("Time saved by Proxy:                  " + (naiveTime - smartTime) + "ms");
        System.out.println("==================================================");
    }

    private static long test(YouTubeManager manager) {
        long startTime = System.currentTimeMillis();

        // 1. Initial fetch of popular videos
        manager.renderPopularVideos();

        // 2. Initial fetch of specific video
        manager.renderVideoPage("catzzzzzzzzz");

        // 3. Repeated fetch of popular videos (should hit cache in proxy)
        manager.renderPopularVideos();

        // 4. Fetch another video
        manager.renderVideoPage("dancesvideoo");

        // 5. Repeated fetch of already viewed video (should hit cache in proxy)
        manager.renderVideoPage("catzzzzzzzzz");

        // 6. Fetch a new video
        manager.renderVideoPage("mkuiggsdg333");

        long estimatedTime = System.currentTimeMillis() - startTime;
        System.out.println("Elapsed Time: " + estimatedTime + "ms");
        return estimatedTime;
    }
}
