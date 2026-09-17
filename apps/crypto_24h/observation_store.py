class ObservationStore:
    def __init__(self):
        """
        Manages a continuous UTC 1m stream.
        Strict ordering, deduplication, no session reset.
        """
        self.observations = []
        self._timestamp_set = set()
        
    def add(self, timestamp, open_px, high_px, low_px, close_px, volume):
        """
        Add a 1m observation. Ensure it is chronologically ordered and not a duplicate.
        """
        if timestamp in self._timestamp_set:
            return False # Duplicate
            
        if self.observations and timestamp <= self.observations[-1]["timestamp"]:
            raise ValueError(f"Out of order observation: {timestamp} <= {self.observations[-1]['timestamp']}")
            
        obs = {
            "timestamp": timestamp,
            "open": open_px,
            "high": high_px,
            "low": low_px,
            "close": close_px,
            "volume": volume
        }
        self.observations.append(obs)
        self._timestamp_set.add(timestamp)
        return True
        
    def get_recent(self, count):
        """
        Return the most recent `count` observations.
        """
        return self.observations[-count:] if count > 0 else []

    def get_all(self):
        return self.observations
