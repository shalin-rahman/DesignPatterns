/**
 * Video - Data model representing a YouTube video.
 *
 * Role:
 *   - Domain / Transfer Object: Holds metadata and content payload.
 */
public class Video {
    public final String id;
    public final String title;
    public final String data;

    public Video(String id, String title) {
        this.id = id;
        this.title = title;
        this.data = "Random video payload content for ID: " + id;
    }

    @Override
    public String toString() {
        return "Video{id='" + id + "', title='" + title + "'}";
    }
}
