import XCTest

@main
class ARWRunner: NSObject {
    static func main() {
        // Create an observer that prints to stdout
        let observer = XCTestObservationCenter.shared
        observer.addTestObserver(TestPrinter())
        
        let suite = XCTestSuite.default
        // Add our test case
        suite.addTest(XCTestSuite(forTestCaseClass: ARWServiceTests.self))
        
        suite.run()
    }
}

class TestPrinter: NSObject, XCTestObservation {
    func testCase(_ testCase: XCTestCase, didFailWithDescription description: String, inFile filePath: String?, atLine lineNumber: Int) {
        print("❌ \(testCase.name) FAILED: \(description)")
    }

    func testCaseDidFinish(_ testCase: XCTestCase) {
        if testCase.testRun?.hasSucceeded == true {
            print("✅ \(testCase.name) PASSED")
        }
    }
}
