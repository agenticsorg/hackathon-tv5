import Foundation
import CloudKit
@testable import VibeCheck

// MARK: - Mocks

class MockCKDatabase: CKDatabaseProtocol {
    var savedRecords: [CKRecord] = []
    var recordsToReturn: [CKRecord] = []
    var errorToReturn: Error?
    
    func save(_ record: CKRecord, completionHandler: @escaping (CKRecord?, Error?) -> Void) {
        if let error = errorToReturn {
            completionHandler(nil, error)
            return
        }
        savedRecords.append(record)
        // Simulate async
        DispatchQueue.global().async {
            completionHandler(record, nil)
        }
    }
    
    func perform(_ query: CKQuery, inZoneWith zoneID: CKRecordZone.ID?, completionHandler: @escaping ([CKRecord]?, Error?) -> Void) {
        if let error = errorToReturn {
            completionHandler(nil, error)
            return
        }
        DispatchQueue.global().async {
            completionHandler(self.recordsToReturn, nil)
        }
    }
}

class MockCKContainer: CKContainerProtocol {
    let mockPrivateDB = MockCKDatabase()
    
    var privateCloudDatabaseProtocol: CKDatabaseProtocol {
        return mockPrivateDB
    }
}
