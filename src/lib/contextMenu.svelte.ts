export interface ContextMenuItem {
  label: string;
  icon?: any;
  action: () => void;
  danger?: boolean;
  disabled?: boolean;
  separator?: boolean;
}

class ContextMenuManager {
  isOpen = $state(false);
  x = $state(0);
  y = $state(0);
  items = $state<ContextMenuItem[]>([]);

  open(e: MouseEvent, items: ContextMenuItem[]) {
    e.preventDefault();
    e.stopPropagation();

    const menuWidth = 240;
    const menuHeight = items.length * 36 + 20;
    const winWidth = typeof window !== 'undefined' ? window.innerWidth : 1200;
    const winHeight = typeof window !== 'undefined' ? window.innerHeight : 800;

    let posX = e.clientX;
    let posY = e.clientY;

    if (posX + menuWidth > winWidth) {
      posX = Math.max(10, winWidth - menuWidth - 10);
    }
    if (posY + menuHeight > winHeight) {
      posY = Math.max(10, winHeight - menuHeight - 10);
    }

    this.x = posX;
    this.y = posY;
    this.items = items;
    this.isOpen = true;
  }

  close() {
    this.isOpen = false;
    this.items = [];
  }
}

export const contextMenu = new ContextMenuManager();
