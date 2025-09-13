import React from 'react';

/**
 * FocusNotification Component
 *
 * Props:
 * - isVisible: boolean            // show/hide
 * - topOffset: string             // e.g. "15vh"
 * - text: string                  // e.g. "Focus Mode ON"
 * - className: string             // extra wrapper classes
 * - style: object                 // extra wrapper styles
 */
const FocusNotification = ({
  isVisible = false,
  topOffset = '15vh',
  text = 'Monitoring ON',
  className = '',
  style = {},
}) => {
  if (!isVisible) return null;

  return (
    <div
      className={`fixed left-1/2 z-50 pointer-events-none transform -translate-x-1/2 ${className}`}
      style={{
        top: topOffset,
        transition: 'top 0.3s cubic-bezier(0.25,0.46,0.45,0.94)',
        ...style,
      }}
    >
      <div
        className="flex items-center px-3 py-1.5 sm:px-4 sm:py-2 md:px-5 md:py-2.5 lg:px-6 lg:py-3 xl:px-8 xl:py-4 rounded-full gap-2 sm:gap-2.5 md:gap-3 lg:gap-3.5 xl:gap-4"
        style={{
          background: 'hsl(var(--secondary))',
          color: 'hsl(var(--primary))',
          borderRadius: '49.846px',
        }}
      >
        {/* eye icon - responsive sizing */}
        <svg 
          xmlns="http://www.w3.org/2000/svg" 
          className="w-4 h-3 sm:w-5 sm:h-4 md:w-6 md:h-5 lg:w-7 lg:h-6 xl:w-8 xl:h-7 flex-shrink-0"
          viewBox="0 0 29 22" 
          fill="none"
        >
          <path d="M1.36008 11.5253C1.17425 11.231 1.08132 11.0839 1.0293 10.8569C0.990233 10.6865 0.990233 10.4177 1.0293 10.2472C1.08132 10.0203 1.17425 9.8732 1.36008 9.57886C2.89579 7.14723 7.46697 1 14.4332 1C21.3994 1 25.9706 7.14723 27.5063 9.57886C27.6921 9.8732 27.7851 10.0203 27.8371 10.2472C27.8761 10.4177 27.8761 10.6865 27.8371 10.8569C27.7851 11.0839 27.6921 11.231 27.5063 11.5253C25.9706 13.957 21.3994 20.1042 14.4332 20.1042C7.46697 20.1042 2.89579 13.957 1.36008 11.5253Z" stroke="hsl(var(--primary))" strokeWidth="1.965" strokeLinecap="round" strokeLinejoin="round"/>
          <path d="M14.5 14.2216C16.761 14.2216 18.5938 12.3812 18.5938 10.1108C18.5938 7.84041 16.761 6 14.5 6C12.239 6 10.4062 7.84041 10.4062 10.1108C10.4062 12.3812 12.239 14.2216 14.5 14.2216Z" stroke="hsl(var(--primary))" strokeWidth="1.965" strokeLinecap="round" strokeLinejoin="round"/>
        </svg>

        <span
          className="font-medium text-xs sm:text-sm md:text-base lg:text-lg xl:text-xl body-text tracking-box"
          style={{ lineHeight: 1 }}
        >
          {text}
        </span>
      </div>
    </div>
  );
};

export default FocusNotification;
