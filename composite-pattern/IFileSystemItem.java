import java.util.List;

/**
 * Component shared by files and folders. Clients can search either without
 * knowing which concrete type they received.
 */
public interface IFileSystemItem {
    String getName();

    int getSize();

    /** Prints this item and its contents as an indented tree. */
    default void display() {
        display(0);
    }

    /** Internal tree-rendering operation shared by leaves and composites. */
    void display(int indentationLevel);

    /** Returns matching files, including this file when called on a File. */
    List<File> search(String keyword);
}
