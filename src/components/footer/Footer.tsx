import React from "react";

import ModelSelector from "../model-selector";

const Footer: React.FC = () => {
  return (
    <div className="w-full border-t border-mid-gray/20 pt-3">
      <div className="flex items-center text-xs px-4 pb-3 text-text/60">
        <ModelSelector />
      </div>
    </div>
  );
};

export default Footer;
