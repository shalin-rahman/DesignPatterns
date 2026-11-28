/**
 * Concrete intermediate container handler that can display a wiki help page.
 */
public class Dialog extends Container {

    private final String wikiPageUrl;

    public Dialog() {
        this.wikiPageUrl = null;
    }

    public Dialog(String wikiPageUrl) {
        this.wikiPageUrl = wikiPageUrl;
    }

    /**
     * Opens the wiki documentation page if available; otherwise forwards to parent container.
     */
    @Override
    public void showHelp() {
        if (wikiPageUrl != null) {
            System.out.println("[Dialog] Opening wiki page for context-specific help: " + wikiPageUrl);
        } else {
            System.out.println("[Dialog] No wiki page — forwarding to " +
                    (container != null ? container.getClass().getSimpleName() : "nobody (end of chain)") + "...");
            if (container != null) {
                container.showHelp();
            } else {
                System.out.println("[Dialog] End of chain — no help available.");
            }
        }
    }

    public String getWikiPageUrl() {
        return wikiPageUrl;
    }
}
