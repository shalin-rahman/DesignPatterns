package AbstructFactory_EduSphre;

// Concrete Product A2
public class OfflineLecture implements Lecture {
    @Override
    public void deliver() {
        System.out.println("Delivering lecture in the classroom");
    }

    @Override
    public void shareMaterials() {
        System.out.println("Distributing printed handouts to students");
    }
}
