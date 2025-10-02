import React from 'react'
import { AppLayout } from "@/components/app-layout"
import { PageHeader } from "@/components/page-header"
import { DataTable } from "@/components/node-table"

export default function page() {
  return (
      <AppLayout>
        <PageHeader/>
        <div className='h-full'>
            <DataTable/>
        </div>
      </AppLayout>
  )
}
