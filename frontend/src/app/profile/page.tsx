"use client";
import React from "react";

import { AppLayout } from "@/components/app-layout";
import { ArrowLeftIcon } from "@/public/assets/icons/arrow-left";
import Link from "next/link";
import Image from "next/image";
import { MemberTable } from "@/components/member-table";
import { useSession } from "next-auth/react";
import CancelModal from "@/public/assets/images/cancel-modal.svg";

export default function ProfilePage() {
  const { data: session } = useSession();
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string>("");
  const [user, setUser] = React.useState<{ name: string; email: string; role: string } | null>(null);
  const [memberCount, setMemberCount] = React.useState<number | null>(null);
  const [showAddMember, setShowAddMember] = React.useState(false);

  // New: collected emails shown as chips
  const [emails, setEmails] = React.useState<string[]>([]);
  const [emailInput, setEmailInput] = React.useState<string>("");

  // Accepts Blessing@gmail.com and also @yahoo.com-like forms (kept your original regex)
  const emailRegex = React.useMemo(() => /^(?:[^\s@]+)?@[^\s@]+\.[^\s@]+$/i, []);

  React.useEffect(() => {
    const fetchUser = async () => {
      if (!session?.user?.id) return;
      try {
        setIsLoading(true);
        setError("");
        const res = await fetch(`/api/profile?id=${encodeURIComponent(session.user.id)}`);
        if (!res.ok) {
          const txt = await res.text();
          throw new Error(txt || "Failed to fetch user details");
        }
        const data = await res.json();
        const apiUser = data?.data;
        setUser({
          name: session.user.name || apiUser?.username || "",
          email: session.user.email || apiUser?.email || "",
          role: session.user.role || apiUser?.role || "",
        });
        // Fetch member count in parallel after session ensures token
        try {
          const usersRes = await fetch(`/api/account/users?per_page=1&page=1`);
          if (usersRes.ok) {
            const usersData = await usersRes.json();
            const total = usersData?.pagination?.total_items ?? usersData?.data?.total ?? usersData?.data?.items?.length ?? null;
            setMemberCount(typeof total === "number" ? total : null);
          }
        } catch {}
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to fetch user");
      } finally {
        setIsLoading(false);
      }
    };
    fetchUser();
  }, [session?.user?.id, session?.user?.name, session?.user?.email, session?.user?.role]);

  // Helper to parse a string containing one or many emails separated by commas,
  // add valid ones to the emails array and keep any trailing partial input.
  const addEmailsFromString = (value: string) => {
    const parts = value.split(",").map((s) => s.trim()).filter(Boolean);
    if (parts.length) {
      const valids = parts.filter((p) => emailRegex.test(p));
      if (valids.length) {
        setEmails((prev) => {
          const set = new Set(prev);
          valids.forEach((v) => set.add(v));
          return Array.from(set);
        });
      }
    }
    // Determine remainder (if user typed "a@b.com, pa" keep "pa" in the input)
    const remainder = value.endsWith(",") ? "" : (value.split(",").pop() ?? "").trim();
    setEmailInput(remainder);
  };

  const removeEmail = (index: number) => {
    setEmails((prev) => prev.filter((_, i) => i !== index));
  };

  // Whether there is at least one valid item ready to submit
  const hasValidInput = emailInput.trim() && emailRegex.test(emailInput.trim());
  const canSubmit = emails.length > 0 || !!hasValidInput;

  return (
    <AppLayout>
      {/* Breadcrumb */}
      <div className="flex items-center gap-2 text-sm mb-6">
        <span className="text-grey-accent">
          <Link href="/channels">Overview</Link>
        </span>
        <span className="text-grey-accent">&gt;</span>

        <span className="text-blue-primary font-medium">Profile</span>
      </div>

      {/* Back Button */}
      <div className="h-fit">
        <Link
          href="/overview"
          className="flex items-center gap-2 font-medium w-fit mb-4 pl-0 h-auto text-grey-dark text-sm hover:text-grey-dark"
        >
          <ArrowLeftIcon className="h-4 w-4 text-grey-dark" />
          Back
        </Link>
      </div>

      {/* Page Title */}
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-3xl font-medium text-grey-dark ">Profile</h1>
        <button
          onClick={() => setShowAddMember(true)}
          className="bg-blue-primary text-white px-4 py-2 rounded-[50px] flex justify-center items-center gap-4"
        >
          <Image
            src="/add-member.svg"
            alt="Add Member"
            width={20}
            height={20}
          />
          <span>Add Member</span>
        </button>
      </div>

      {/* User Details Section */}
      <div className="bg-white rounded-xl border p-6 mb-8">
        <h2 className="text-base font-medium text-grey-dark mb-6">
          User Details
        </h2>
        {isLoading ? (
          <div className="text-sm text-grey-accent">Loading user...</div>
        ) : error ? (
          <div className="text-sm text-red-600">{error}</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <div>
                <div className="text-sm text-grey-accent mb-1">Name</div>
                <div className="text-base font-medium text-maya-blue">{user?.name || "-"}</div>
              </div>

              <div>
                <div className="text-sm text-grey-accent mb-1 mt-4">Email</div>
                <div className="text-base font-medium text-maya-blue">{user?.email || "-"}</div>
              </div>
            </div>

            <div>
              <div>
                <div className="text-sm text-grey-accent mb-1">Role</div>
                <div className="text-base font-medium text-maya-blue">{user?.role || "-"}</div>
              </div>

              <div>
                <div className="text-sm text-grey-accent mb-1 mt-4">Member</div>
                <div className="text-base font-medium text-maya-blue">{memberCount ?? "-"}</div>
              </div>
            </div>
          </div>
        )}
      </div>

      <MemberTable />

      {showAddMember && (
        <div className="fixed inset-0 z-[9999] flex items-center justify-center">
          {/* Overlay */}
          <div className="absolute inset-0 bg-black/20" onClick={() => setShowAddMember(false)} />

          {/* Modal */}
          <div className="relative bg-white w-full max-w-[520px] rounded-xl shadow-xl border p-6 mx-4">
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-medium text-grey-dark">Add Member</h3>
              <button
                type="button"
                aria-label="Close"
                onClick={() => setShowAddMember(false)}
                className="h-6 w-6 rounded-full text-grey-dark hover:bg-gray-100"
              >
                <Image src={CancelModal} alt="Cancel" width={20} height={20} />
              </button>
            </div>

            {/* Field */}
            <div className="space-y-2">
              <label className="text-sm text-grey-accent">Email</label>
              <input
                type="text"
                placeholder="Enter email"
                value={emailInput}
                onChange={(e) => {
                  const val = e.target.value;
                  if (val.includes(",")) {
                    addEmailsFromString(val);
                  } else {
                    setEmailInput(val);
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    if (emailInput.trim()) addEmailsFromString(emailInput);
                  }
                }}
                className="w-full rounded-lg border border-[#D4D4D4] bg-white px-3 py-3 text-sm outline-none text-[#000000] font-[500]"
              />
              <p className="text-[15px] text-[#000000] font-[500]">Add a comma after an email address to add more.</p>

              {/* Chips for added emails */}
              {emails.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-2">
                  {emails.map((em, idx) => (
                    <div key={`${em}-${idx}`} className="flex items-center gap-2 px-3 py-1 bg-[#E7E7E7] rounded-full text-sm text-[#000000] font-[500]">
                      <span className="truncate max-w-[220px] text-black">{em}</span>
                      <button
                        type="button"
                        onClick={(ev) => {
                          ev.stopPropagation();
                          removeEmail(idx);
                        }}
                        className="h-5 w-5 flex items-center justify-center rounded-full hover:bg-gray-200 text-gray-600"
                        aria-label={`Remove ${em}`}
                      >
                        <Image src={CancelModal} alt="Cancel" width={15} height={15} />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="mt-6">
              <button
                type="button"
                disabled={!canSubmit}
                className={`w-full rounded-full px-4 py-3 text-sm font-medium ${canSubmit ? "bg-blue-primary text-white" : "bg-gray-300 text-gray-600 cursor-not-allowed"}`}
                onClick={() => {
                  // Ensure any remaining typed valid email is added before closing
                  if (emailInput.trim() && emailRegex.test(emailInput.trim())) {
                    addEmailsFromString(emailInput);
                  }
                  if (!canSubmit && emails.length === 0) return;
                  // TODO: submit `emails` to your API here.
                  setShowAddMember(false);
                  setEmailInput("");
                  setEmails([]);
                }}
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}
    </AppLayout>
  );
}
