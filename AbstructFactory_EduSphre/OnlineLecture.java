package AbstructFactory_EduSphre;

// Concrete Product A1
public class OnlineLecture implements Lecture {
    @Override
    public void deliver() {
        System.out.println("Delivering lecture via live video session");
    }

    @Override
    public void shareMaterials() {
        System.out.println("Uploading recording and slides to the course portal");
    }
}
