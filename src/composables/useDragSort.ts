import { ref, onUnmounted, type Ref } from "vue";

export interface DragSortOptions {
  listRef: Ref<HTMLElement | null>;
  itemSelector: string;
  ghostClass: string;
  skipSelector?: string;
  gap?: number;
  onDrop: (fromIndex: number, toIndex: number) => void;
}

export function useDragSort({
  listRef,
  itemSelector,
  ghostClass,
  skipSelector,
  gap = 0,
  onDrop,
}: DragSortOptions) {
  const dragIndex = ref<number | null>(null);
  const dropTarget = ref<number | null>(null);
  const itemHeight = ref(0);

  let startY = 0;
  let dragging = false;
  let ghostEl: HTMLElement | null = null;
  let offsetX = 0;
  let offsetY = 0;
  let activeMoveHandler: ((ev: MouseEvent) => void) | null = null;
  let activeUpHandler: (() => void) | null = null;

  function getItemStyle(index: number) {
    if (dragIndex.value === null || dropTarget.value === null || !itemHeight.value) return {};

    const from = dragIndex.value;
    const to = dropTarget.value;
    let offset = 0;

    if (from < to) {
      if (index > from && index <= to) offset = -1;
    } else if (from > to) {
      if (index >= to && index < from) offset = 1;
    }

    if (offset === 0) return {};
    return {
      transform: `translateY(${offset * itemHeight.value}px)`,
      transition: "transform 0.2s ease",
    };
  }

  function createGhost(el: HTMLElement, e: MouseEvent) {
    const rect = el.getBoundingClientRect();
    offsetX = e.clientX - rect.left;
    offsetY = e.clientY - rect.top;

    ghostEl = el.cloneNode(true) as HTMLElement;
    ghostEl.className = ghostClass;
    ghostEl.style.width = rect.width + "px";
    ghostEl.style.position = "fixed";
    ghostEl.style.left = rect.left + "px";
    ghostEl.style.top = rect.top + "px";
    ghostEl.style.zIndex = "999";
    ghostEl.style.pointerEvents = "none";
    ghostEl.style.margin = "0";
    document.body.appendChild(ghostEl);
  }

  function removeGhost() {
    if (ghostEl) {
      ghostEl.remove();
      ghostEl = null;
    }
  }

  function moveGhost(clientX: number, clientY: number) {
    if (!ghostEl) return;
    ghostEl.style.left = (clientX - offsetX) + "px";
    ghostEl.style.top = (clientY - offsetY) + "px";
  }

  function cleanupDragListeners() {
    if (activeMoveHandler) {
      document.removeEventListener("mousemove", activeMoveHandler);
      activeMoveHandler = null;
    }
    if (activeUpHandler) {
      document.removeEventListener("mouseup", activeUpHandler);
      activeUpHandler = null;
    }
  }

  function onMouseDown(e: MouseEvent, index: number) {
    if (skipSelector && (e.target as HTMLElement).closest(skipSelector)) return;
    startY = e.clientY;
    dragging = false;
    dragIndex.value = index;

    const onMove = (ev: MouseEvent) => {
      if (!dragging && Math.abs(ev.clientY - startY) > 5) {
        dragging = true;
        dragIndex.value = index;
        if (listRef.value) {
          const first = listRef.value.querySelector<HTMLElement>(itemSelector);
          if (first) itemHeight.value = first.offsetHeight + gap;
        }
        const items = listRef.value?.querySelectorAll<HTMLElement>(itemSelector);
        if (items?.[index]) createGhost(items[index], ev);
      }
      if (!dragging || !listRef.value) return;

      moveGhost(ev.clientX, ev.clientY);

      const items = listRef.value.querySelectorAll<HTMLElement>(itemSelector);
      let target: number | null = null;
      for (let i = 0; i < items.length; i++) {
        const rect = items[i].getBoundingClientRect();
        if (ev.clientY < rect.bottom) {
          target = i;
          break;
        }
      }
      if (target === null && items.length > 0) {
        target = items.length - 1;
      }
      dropTarget.value = target === dragIndex.value ? null : target;
    };

    const onUp = () => {
      if (dragging && dragIndex.value !== null && dropTarget.value !== null) {
        onDrop(dragIndex.value, dropTarget.value);
      }
      removeGhost();
      dragIndex.value = null;
      dropTarget.value = null;
      itemHeight.value = 0;
      dragging = false;
      cleanupDragListeners();
    };

    activeMoveHandler = onMove;
    activeUpHandler = onUp;
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  }

  onUnmounted(() => {
    dragging = false;
    removeGhost();
    cleanupDragListeners();
  });

  return { dragIndex, dropTarget, getItemStyle, onMouseDown, isDragging: () => dragging };
}
