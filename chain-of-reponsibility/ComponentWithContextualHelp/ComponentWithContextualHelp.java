/**
 * Handler interface declaring the execution contract for the Chain of Responsibility.
 */
public interface ComponentWithContextualHelp {

    /**
     * Shows contextual help or forwards the request to the next handler in the chain.
     */
    void showHelp();
}
