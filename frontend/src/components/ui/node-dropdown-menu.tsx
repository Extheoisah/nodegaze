import React from "react";
import { Button } from "./button";
import { createPortal } from "react-dom";
import Image from "next/image";
import plug from "../../../public/plug.svg";
import Delete from "../../../public/delete.svg";
import sync from "../../../public/sync.svg";

export function NodeActions({
  nodeId,
  openId,
  setOpenId,
}: {
  nodeId: number;
  openId: number | null;
  setOpenId: (id: number | null) => void;
}) {
  const isOpen = openId === nodeId;
  const buttonRef = React.useRef<HTMLButtonElement>(null);
  const [dropdownPos, setDropdownPos] = React.useState<{
    top: number;
    left: number;
  }>({ top: 0, left: 0 });

  // Set dropdown position relative to button
  React.useEffect(() => {
    if (isOpen && buttonRef.current) {
      const rect = buttonRef.current.getBoundingClientRect();
      setDropdownPos({
        top: rect.bottom + window.scrollY + 4, // 4px margin
        left: rect.left + window.scrollX - 200 + rect.width, // align right edge, adjust as needed
      });
    }
  }, [isOpen]);

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setOpenId(isOpen ? null : nodeId);
  };

  const handleConnect = (e: React.MouseEvent) => {
    e.stopPropagation();
    console.log("Connect clicked for node:", nodeId);
    setOpenId(null);
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    console.log("Delete clicked for node:", nodeId);
    setOpenId(null);
  };

  const handleSync = (e: React.MouseEvent) => {
    e.stopPropagation();
    console.log("Sync clicked for node:", nodeId);
    setOpenId(null);
  };

  React.useEffect(() => {
    if (!isOpen) return;
    const handleClick = () => setOpenId(null);
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  }, [isOpen, setOpenId]);

  return (
    <>
      <Button
        ref={buttonRef}
        variant="outline"
        className={`h-8 w-8 p-0 rounded-[8px] cursor-pointer transition-colors ${
          isOpen
            ? "bg-blue-500 text-white border-blue-500 hover:bg-blue-600"
            : "text-grey-dark hover:bg-gray-50"
        }`}
        onClick={handleToggle}
        type="button"
        tabIndex={0}
      >
        <div className="flex flex-col items-center justify-center space-y-0.5">
          <div
            className={`w-1 h-1 rounded-full ${
              isOpen ? "bg-white" : "bg-current"
            }`}
          ></div>
          <div
            className={`w-1 h-1 rounded-full ${
              isOpen ? "bg-white" : "bg-current"
            }`}
          ></div>
          <div
            className={`w-1 h-1 rounded-full ${
              isOpen ? "bg-white" : "bg-current"
            }`}
          ></div>
        </div>
      </Button>

      {isOpen &&
        typeof window !== "undefined" &&
        createPortal(
          <div
            className="absolute z-[9999999] w-[250px] bg-white border border-gray-200 rounded-lg shadow-lg"
            style={{
              top: dropdownPos.top,
              right: 60,
              //   left: dropdownPos.left,
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <button
              onClick={handleConnect}
              className="w-full px-3 py-2 text-left text-sm hover:bg-gray-50 rounded-t-lg flex items-center space-x-2"
              type="button"
            >
              <div className="flex space-x-2">
                <Image src={plug} alt="" />
                <span className="text-black font-[500]">Connect</span>
              </div>
            </button>
            <button
              onClick={handleDelete}
              className="w-full px-3 py-2 text-left text-sm hover:bg-gray-50 flex items-center space-x-2"
              type="button"
            >
              <div className="flex space-x-2">
                {" "}
                <Image src={Delete} alt="" />
                <span className="text-black font-[500]">Delete</span>
              </div>
            </button>
            <button
              onClick={handleSync}
              className="w-full px-3 py-2 text-left text-sm hover:bg-gray-50 rounded-b-lg flex items-center space-x-2"
              type="button"
            >
              <div className="flex space-x-2">
                <Image src={sync} alt="" />
                <span className="text-black font-[500]">Sync</span>
              </div>
            </button>
          </div>,
          document.body
        )}
    </>
  );
}
