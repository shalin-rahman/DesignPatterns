package AbstructFactory_EduSphre;

// Abstract Factory: one method per product in the family
public interface CourseComponentFactory {
    Lecture createLecture();
    Assessment createAssessment();
}
