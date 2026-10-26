import java.util.List;

/** Builds and demonstrates the searchable composite file system. */
public class Main {
    public static void main(String[] args) {
        Folder root = new Folder("root");
        Folder docs = new Folder("docs");
        Folder reports = new Folder("reports");

        File readme = new File("readme.txt", 4, "Welcome to the design notes.");
        File notes = new File("notes.txt", 3,
                "Composite treats files and folders uniformly.");
        File q1 = new File("q1.txt", 10, "Design review scheduled.");
        File mainJava = new File("Main.java", 6, "composite demo");
        File todo = new File("todo.txt", 1, "Finish design document.");

        reports.add(q1);
        docs.add(readme);
        docs.add(notes);
        docs.add(reports);
        root.add(docs);
        root.add(mainJava);
        root.add(todo);

        //File system
        System.out.println("=== File system tree ===");
        root.display();
        System.out.println();

        System.out.println("root size: " + root.getSize() + " KB (expected 24 KB)");
        System.out.println("docs size: " + docs.getSize() + " KB (expected 17 KB)");
        System.out.println();

        // The same client method accepts the component for root, folder, or file.
        runSearch("root", root, "design");
        runSearch("docs", docs, "design");
        runSearch("todo.txt", todo, "design");
        runSearch("todo.txt", todo, "sales");
        runSearch("root", root, "composite");

        //Add a file to docs
        System.out.println("\n=== Add a file to docs ===");
        File extra = new File("design-extra.txt", 2, "Design details and decisions.");
        docs.add(extra);
        System.out.println("root size after add: " + root.getSize() + " KB (expected 26 KB)");
        System.out.println("docs size after add: " + docs.getSize() + " KB (expected 19 KB)");
        runSearch("root", root, "design");

        System.out.println("=== Remove the added file ===");
        System.out.println("removed: " + docs.remove(extra));
        System.out.println("root size after remove: " + root.getSize() + " KB (expected 24 KB)");
        System.out.println("docs size after remove: " + docs.getSize() + " KB (expected 17 KB)");
    }

    /** Client code depends only on IFileSystemItem, never File or Folder. */
    private static void runSearch(String location, IFileSystemItem item, String keyword) {
        List<File> matches = item.search(keyword);
        System.out.print(location + ".search(\"" + keyword + "\"): ");
        if (matches.isEmpty()) {
            System.out.println("no results");
            return;
        }

        for (int i = 0; i < matches.size(); i++) {
            if (i > 0) {
                System.out.print(", ");
            }
            System.out.print(matches.get(i).getName());
        }
        System.out.println();
    }
}
