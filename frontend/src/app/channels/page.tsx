import { AppLayout } from "@/components/app-layout"
import { PageHeader } from "@/components/page-header"
import { DataTable } from "@/components/base-table"



export default function Dashboard() {
  return (
    <AppLayout>
      <PageHeader />

      <div className="h-full">
        <DataTable />
      </div>
    </AppLayout>
  )
}
