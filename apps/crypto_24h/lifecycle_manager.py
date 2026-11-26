class ActivePosition:
    def __init__(self, brief, entry_px, horizon=300, e4_threshold=0.0050):
        self.brief = brief
        self.entry_px = entry_px
        self.direction = brief.direction
        
        self.horizon_bars = horizon
        self.bars_elapsed = 0
        self.e4_threshold = e4_threshold
        
        self.is_closed = False
        self.exit_px = None
        self.exit_reason = None
        
    def update(self, current_obs):
        if self.is_closed:
            return
            
        self.bars_elapsed += 1
        px = current_obs["close"]
        
        # Check E4 Trigger
        ret = (px - self.entry_px)/self.entry_px if self.direction == "LONG" else (self.entry_px - px)/self.entry_px
        if ret >= self.e4_threshold:
            self.is_closed = True
            self.exit_px = px
            self.exit_reason = "E4_TRIGGER"
            return
            
        # Check Horizon
        if self.bars_elapsed >= self.horizon_bars:
            self.is_closed = True
            self.exit_px = px
            self.exit_reason = "HORIZON_REACHED"
            return

class LifecycleManager:
    def __init__(self):
        """
        Manages the lifecycle of active positions.
        V1: E4-0.50 + continuous horizon. No adaptive horizon.
        """
        self.active_positions = []
        self.closed_positions = []
        
    def add_position(self, brief, entry_px):
        pos = ActivePosition(brief, entry_px)
        self.active_positions.append(pos)
        
    def update(self, current_obs):
        for pos in self.active_positions:
            pos.update(current_obs)
            
        # Move closed to history
        new_active = []
        for pos in self.active_positions:
            if pos.is_closed:
                self.closed_positions.append(pos)
            else:
                new_active.append(pos)
        self.active_positions = new_active
