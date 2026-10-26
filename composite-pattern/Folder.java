import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Objects;

/** Composite: a folder contains any number of files and other folders. */
public class Folder implements IFileSystemItem {
    private final String name;
    private final List<IFileSystemItem> children = new ArrayList<>();

    public Folder(String name) {
        this.name = Objects.requireNonNull(name, "name");
    }

    @Override
    public String getName() {
        return name;
    }

    /** The folder's size is calculated from its current descendants. */
    @Override
    public int getSize() {
        int totalSize = 0;
        for (IFileSystemItem child : children) {
            totalSize += child.getSize();
        }
        return totalSize;
    }

    public void add(IFileSystemItem item) {
        children.add(Objects.requireNonNull(item, "item"));
    }

    public boolean remove(IFileSystemItem item) {
        return children.remove(item);
    }

    @Override
    public void display(int indentationLevel) {
        System.out.println(indentation(indentationLevel) + name + "/ ("
                + getSize() + " KB)");
        for (IFileSystemItem child : children) {
            child.display(indentationLevel + 1);
        }
    }

    /**
     * Each child performs its own search; combining the results makes nested
     * folders work recursively without checking whether a child is a file.
     */
    @Override
    public List<File> search(String keyword) {
        Objects.requireNonNull(keyword, "keyword");
        String normalizedKeyword = keyword.toLowerCase(Locale.ROOT);
        List<File> matches = new ArrayList<>();
        for (IFileSystemItem child : children) {
            matches.addAll(child.search(normalizedKeyword));
        }
        return Collections.unmodifiableList(matches);
    }

    private static String indentation(int level) {
        StringBuilder indentation = new StringBuilder();
        for (int i = 0; i < level; i++) {
            indentation.append("  ");
        }
        return indentation.toString();
    }
}
