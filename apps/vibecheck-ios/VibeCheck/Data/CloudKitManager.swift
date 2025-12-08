import Foundation
import CloudKit

import Foundation
import CloudKit

// MARK: - Protocols for Dependency Injection

protocol CKDatabaseProtocol {
    func save(_ record: CKRecord, completionHandler: @escaping (CKRecord?, Error?) -> Void)
    func perform(_ query: CKQuery, inZoneWith zoneID: CKRecordZone.ID?, completionHandler: @escaping ([CKRecord]?, Error?) -> Void)
}

protocol CKContainerProtocol {
    var privateCloudDatabaseProtocol: CKDatabaseProtocol { get }
}

// MARK: - Data Model

struct FamilyMemberVibe: Identifiable {
    let id: UUID
    let userID: String
    let vibeKeyword: String
    let vibeEnergy: String
    let vibeStress: String
    let colorHex: String
    let lastUpdated: Date
    
    // Convert to CKRecord
    func toCKRecord() -> CKRecord {
        let recordID = CKRecord.ID(recordName: userID) // One record per user
        let record = CKRecord(recordType: "FamilyVibe", recordID: recordID)
        record["user_id"] = userID
        record["vibe_keyword"] = vibeKeyword
        record["vibe_energy"] = vibeEnergy
        record["vibe_stress"] = vibeStress
        record["color_hex"] = colorHex
        record["last_updated"] = lastUpdated
        return record
    }
}

// MARK: - CloudKit Manager

class CloudKitManager {
    static let shared = CloudKitManager()
    
    // Dependency Injection for Testing
    private let container: CKContainerProtocol
    private let database: CKDatabaseProtocol
    
    init(container: CKContainerProtocol = CKContainer.default()) {
        self.container = container
        self.database = container.privateCloudDatabaseProtocol // Using Private DB for Hackathon simplicity
    }
    
    func publishVibe(_ vibe: FamilyMemberVibe, completion: @escaping (Result<Void, Error>) -> Void) {
        let record = vibe.toCKRecord()
        
        database.save(record) { savedRecord, error in
            if let error = error {
                completion(.failure(error))
                return
            }
            completion(.success(()))
        }
    }
    
    // Fetch function needed for future tests/features
    func fetchFamilyVibes(completion: @escaping (Result<[FamilyMemberVibe], Error>) -> Void) {
        let predicate = NSPredicate(value: true) // Fetch all family vibes
        let query = CKQuery(recordType: "FamilyVibe", predicate: predicate)
        query.sortDescriptors = [NSSortDescriptor(key: "last_updated", ascending: false)]
        
        database.perform(query, inZoneWith: nil) { records, error in
            if let error = error {
                completion(.failure(error))
                return
            }
            
            guard let records = records else {
                completion(.success([]))
                return
            }
            
            let vibes = records.compactMap { record -> FamilyMemberVibe? in
                guard let userID = record["user_id"] as? String,
                      let vibeKeyword = record["vibe_keyword"] as? String,
                      let vibeEnergy = record["vibe_energy"] as? String,
                      let vibeStress = record["vibe_stress"] as? String,
                      let colorHex = record["color_hex"] as? String,
                      let lastUpdated = record["last_updated"] as? Date else {
                    return nil
                }
                
                return FamilyMemberVibe(
                    id: UUID(), // Using UUID for local Identifiable compliance
                    userID: userID,
                    vibeKeyword: vibeKeyword,
                    vibeEnergy: vibeEnergy,
                    vibeStress: vibeStress,
                    colorHex: colorHex,
                    lastUpdated: lastUpdated
                )
            }
            
            completion(.success(vibes))
        }
    }
}

// MARK: - Extensions for Real CloudKit mapping

extension CKContainer: CKContainerProtocol {
    var privateCloudDatabaseProtocol: CKDatabaseProtocol {
        return self.privateCloudDatabase
    }
}

extension CKDatabase: CKDatabaseProtocol {
    // Protocol requirements match existing methods exactly
    // Explicit conformance is automatic
}
