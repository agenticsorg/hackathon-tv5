/**
 * QTable Tests
 * TDD London School approach
 */

import { QTable } from '../../../src/rl/q-table';
import { QTableEntry } from '../../../src/rl/types';

describe('QTable', () => {
  let qTable: QTable;

  beforeEach(() => {
    qTable = new QTable();
  });

  describe('get', () => {
    it('should return QTableEntry for existing state-action pair', async () => {
      // Arrange
      const entry: QTableEntry = {
        stateHash: '2:3:1',
        contentId: 'content-1',
        qValue: 0.72,
        visitCount: 5,
        lastUpdated: Date.now()
      };
      await qTable.set(entry);

      // Act
      const result = await qTable.get('2:3:1', 'content-1');

      // Assert
      expect(result).toBeDefined();
      expect(result?.qValue).toBe(0.72);
      expect(result?.visitCount).toBe(5);
    });

    it('should return null for non-existent state-action pair', async () => {
      // Act
      const result = await qTable.get('0:0:0', 'content-999');

      // Assert
      expect(result).toBeNull();
    });
  });

  describe('set', () => {
    it('should store QTableEntry', async () => {
      // Arrange
      const entry: QTableEntry = {
        stateHash: '3:2:1',
        contentId: 'content-2',
        qValue: 0.45,
        visitCount: 3,
        lastUpdated: Date.now()
      };

      // Act
      await qTable.set(entry);
      const result = await qTable.get('3:2:1', 'content-2');

      // Assert
      expect(result).toEqual(entry);
    });

    it('should update existing entry', async () => {
      // Arrange
      const entry1: QTableEntry = {
        stateHash: '2:3:1',
        contentId: 'content-1',
        qValue: 0.5,
        visitCount: 5,
        lastUpdated: Date.now()
      };
      await qTable.set(entry1);

      const entry2: QTableEntry = {
        stateHash: '2:3:1',
        contentId: 'content-1',
        qValue: 0.6,
        visitCount: 6,
        lastUpdated: Date.now()
      };

      // Act
      await qTable.set(entry2);
      const result = await qTable.get('2:3:1', 'content-1');

      // Assert
      expect(result?.qValue).toBe(0.6);
      expect(result?.visitCount).toBe(6);
    });
  });

  describe('updateQValue', () => {
    it('should update Q-value and increment visit count', async () => {
      // Arrange
      const entry: QTableEntry = {
        stateHash: '2:3:1',
        contentId: 'content-1',
        qValue: 0.5,
        visitCount: 3,
        lastUpdated: Date.now()
      };
      await qTable.set(entry);

      // Act
      await qTable.updateQValue('2:3:1', 'content-1', 0.65);
      const result = await qTable.get('2:3:1', 'content-1');

      // Assert
      expect(result?.qValue).toBe(0.65);
      expect(result?.visitCount).toBe(4);
    });

    it('should create new entry if not exists', async () => {
      // Act
      await qTable.updateQValue('1:1:2', 'content-3', 0.35);
      const result = await qTable.get('1:1:2', 'content-3');

      // Assert
      expect(result).toBeDefined();
      expect(result?.qValue).toBe(0.35);
      expect(result?.visitCount).toBe(1);
    });
  });

  describe('getStateActions', () => {
    it('should return all Q-table entries for a state', async () => {
      // Arrange
      await qTable.set({
        stateHash: '2:3:1',
        contentId: 'content-1',
        qValue: 0.7,
        visitCount: 5,
        lastUpdated: Date.now()
      });
      await qTable.set({
        stateHash: '2:3:1',
        contentId: 'content-2',
        qValue: 0.4,
        visitCount: 2,
        lastUpdated: Date.now()
      });
      await qTable.set({
        stateHash: '1:1:1',
        contentId: 'content-3',
        qValue: 0.3,
        visitCount: 1,
        lastUpdated: Date.now()
      });

      // Act
      const result = await qTable.getStateActions('2:3:1');

      // Assert
      expect(result).toHaveLength(2);
      expect(result.map(e => e.contentId)).toContain('content-1');
      expect(result.map(e => e.contentId)).toContain('content-2');
    });

    it('should return empty array for unknown state', async () => {
      // Act
      const result = await qTable.getStateActions('9:9:9');

      // Assert
      expect(result).toEqual([]);
    });
  });
});
