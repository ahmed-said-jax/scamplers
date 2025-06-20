/* tslint:disable */
/* eslint-disable */
export enum BlockType {
  Block = 0,
}
export enum ComplianceCommitteeType {
  Ibc = 0,
  Irb = 1,
  Iacuc = 2,
}
export enum FrozenBlockEmbeddingMatrix {
  CarboxymethylCellulose = 0,
  OptimalCuttingTemperatureCompound = 1,
}
export enum InstitutionOrdinalColumn {
  Name = 0,
}
export enum LabOrdinalColumn {
  Name = 0,
}
export enum PersonOrdinalColumn {
  Name = 0,
  Email = 1,
}
export enum Species {
  AmbystomaMexicanum = 0,
  CanisFamiliaris = 1,
  DrosophilaMelanogaster = 2,
  GasterosteusAculeatus = 3,
  HomoSapiens = 4,
  MusMusculus = 5,
  RattusNorvegicus = 6,
  SminthopsisCrassicaudata = 7,
}
export enum TissueType {
  Tissue = 0,
}
export enum UserRole {
  AppAdmin = 0,
  ComputationalStaff = 1,
  BiologyStaff = 2,
}
export class Client {
  free(): void;
  send_new_institution(data: NewInstitution, api_key?: string | null): Promise<Institution>;
  send_new_person(data: NewPerson, api_key?: string | null): Promise<Person>;
  send_new_lab(data: NewLab, api_key?: string | null): Promise<Lab>;
  constructor(backend_url: string, token: string);
  send_new_ms_login(data: NewPerson): Promise<CreatedUser>;
}
export class CommitteeApproval {
  private constructor();
  free(): void;
  institution(): InstitutionHandle;
  committee_type(): ComplianceCommitteeType;
  compliance_identifier(): string;
}
export class CreatedUser {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
  email(): string;
  orcid(): string;
  institution(): Institution;
  roles(): any[];
  api_key(): string;
}
export class EmptyStringError {
  private constructor();
  free(): void;
}
export class Institution {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
}
export class InstitutionHandle {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
}
export class InstitutionOrdering {
  private constructor();
  free(): void;
  static new(): InstitutionOrderingBuilder;
  column: InstitutionOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`InstitutionOrdering`](struct.InstitutionOrdering.html).
 */
export class InstitutionOrderingBuilder {
  private constructor();
  free(): void;
  column(value: InstitutionOrdinalColumn): InstitutionOrderingBuilder;
  descending(value: boolean): InstitutionOrderingBuilder;
  /**
   * Builds a new `InstitutionOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): InstitutionOrdering;
}
export class InstitutionOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class InstitutionQuery {
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  order_by: InstitutionOrdering[];
  pagination: Pagination;
}
export class Lab {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
  delivery_dir(): string;
  pi(): PersonSummary;
  members(): PersonSummary[];
}
export class LabCore {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
  delivery_dir(): string;
  pi(): PersonSummary;
}
export class LabHandle {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
}
export class LabOrdering {
  private constructor();
  free(): void;
  static new(): LabOrderingBuilder;
  column: LabOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`LabOrdering`](struct.LabOrdering.html).
 */
export class LabOrderingBuilder {
  private constructor();
  free(): void;
  column(value: LabOrdinalColumn): LabOrderingBuilder;
  descending(value: boolean): LabOrderingBuilder;
  /**
   * Builds a new `LabOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): LabOrdering;
}
export class LabOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class LabQuery {
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  order_by: LabOrdering[];
  pagination: Pagination;
}
export class LabSummary {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
  delivery_dir(): string;
}
export class LabUpdate {
  private constructor();
  free(): void;
  id(): string;
  name(): NonEmptyString | undefined;
  pi_id(): string;
  delivery_dir(): NonEmptyString | undefined;
  add_members(): string[];
  remove_members(): string[];
  static new(): LabUpdateBuilder;
}
/**
 * Builder for [`LabUpdate`](struct.LabUpdate.html).
 */
export class LabUpdateBuilder {
  private constructor();
  free(): void;
  core(value: LabUpdateCore): LabUpdateBuilder;
  add_members(value: string[]): LabUpdateBuilder;
  remove_members(value: string[]): LabUpdateBuilder;
  /**
   * Builds a new `LabUpdate`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): LabUpdate;
}
export class LabUpdateCore {
  private constructor();
  free(): void;
  id(): string;
  name(): NonEmptyString | undefined;
  pi_id(): string;
  delivery_dir(): NonEmptyString | undefined;
  static new(): LabUpdateCoreBuilder;
}
/**
 * Builder for [`LabUpdateCore`](struct.LabUpdateCore.html).
 */
export class LabUpdateCoreBuilder {
  private constructor();
  free(): void;
  id(value: string): LabUpdateCoreBuilder;
  name(value?: NonEmptyString | null): LabUpdateCoreBuilder;
  pi_id(value?: string | null): LabUpdateCoreBuilder;
  delivery_dir(value?: NonEmptyString | null): LabUpdateCoreBuilder;
  /**
   * Builds a new `LabUpdateCore`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): LabUpdateCore;
}
export class LabUpdateCoreError {
  private constructor();
  free(): void;
  error(): string;
}
export class LabUpdateError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewCommitteeApproval {
  private constructor();
  free(): void;
  static new(): NewCommitteeApprovalBuilder;
}
/**
 * Builder for [`NewCommitteeApproval`](struct.NewCommitteeApproval.html).
 */
export class NewCommitteeApprovalBuilder {
  private constructor();
  free(): void;
  sample_id(value?: string | null): NewCommitteeApprovalBuilder;
  institution_id(value: string): NewCommitteeApprovalBuilder;
  committee_type(value: ComplianceCommitteeType): NewCommitteeApprovalBuilder;
  compliance_identifier(value: NonEmptyString): NewCommitteeApprovalBuilder;
  /**
   * Builds a new `NewCommitteeApproval`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewCommitteeApproval;
}
export class NewCommitteeApprovalError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewInstitution {
  private constructor();
  free(): void;
  static new(): NewInstitutionBuilder;
}
/**
 * Builder for [`NewInstitution`](struct.NewInstitution.html).
 */
export class NewInstitutionBuilder {
  private constructor();
  free(): void;
  id(value: string): NewInstitutionBuilder;
  name(value: NonEmptyString): NewInstitutionBuilder;
  /**
   * Builds a new `NewInstitution`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewInstitution;
}
export class NewInstitutionError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewLab {
  private constructor();
  free(): void;
  static new(): NewLabBuilder;
}
/**
 * Builder for [`NewLab`](struct.NewLab.html).
 */
export class NewLabBuilder {
  private constructor();
  free(): void;
  name(value: NonEmptyString): NewLabBuilder;
  pi_id(value: string): NewLabBuilder;
  delivery_dir(value: NonEmptyString): NewLabBuilder;
  member_ids(value: string[]): NewLabBuilder;
  /**
   * Builds a new `NewLab`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewLab;
}
export class NewLabError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewPerson {
  private constructor();
  free(): void;
  static new(): NewPersonBuilder;
  name: NonEmptyString;
  email: string;
  get orcid(): NonEmptyString | undefined;
  set orcid(value: NonEmptyString | null | undefined);
  institution_id: string;
  get ms_user_id(): string;
  set ms_user_id(value: string | null | undefined);
  roles: any[];
}
/**
 * Builder for [`NewPerson`](struct.NewPerson.html).
 */
export class NewPersonBuilder {
  private constructor();
  free(): void;
  name(value: NonEmptyString): NewPersonBuilder;
  email(value: string): NewPersonBuilder;
  orcid(value?: NonEmptyString | null): NewPersonBuilder;
  institution_id(value: string): NewPersonBuilder;
  ms_user_id(value?: string | null): NewPersonBuilder;
  roles(value: any[]): NewPersonBuilder;
  /**
   * Builds a new `NewPerson`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewPerson;
}
export class NewPersonError {
  private constructor();
  free(): void;
  error(): string;
}
export class NonEmptyString {
  free(): void;
  constructor(s: string);
}
export class Pagination {
  free(): void;
  constructor(limit: bigint, offset: bigint);
  limit: bigint;
  offset: bigint;
}
export class Person {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
  email(): string;
  orcid(): string;
  institution(): Institution;
  roles(): any[];
}
export class PersonCore {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
  email(): string;
  orcid(): string;
  institution(): Institution;
}
export class PersonHandle {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
}
export class PersonOrdering {
  private constructor();
  free(): void;
  static new(): PersonOrderingBuilder;
  column: PersonOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`PersonOrdering`](struct.PersonOrdering.html).
 */
export class PersonOrderingBuilder {
  private constructor();
  free(): void;
  column(value: PersonOrdinalColumn): PersonOrderingBuilder;
  descending(value: boolean): PersonOrderingBuilder;
  /**
   * Builds a new `PersonOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonOrdering;
}
export class PersonOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class PersonQuery {
  free(): void;
  constructor();
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  get email(): string;
  set email(value: string | null | undefined);
  get orcid(): string;
  set orcid(value: string | null | undefined);
  get ms_user_id(): string;
  set ms_user_id(value: string | null | undefined);
  order_by: PersonOrdering[];
  pagination: Pagination;
}
export class PersonSummary {
  private constructor();
  free(): void;
  id(): string;
  link(): string;
  name(): string;
  email(): string;
  orcid(): string;
}
export class PersonUpdate {
  private constructor();
  free(): void;
  grant_roles(): any[];
  revoke_roles(): any[];
  id(): string;
  name(): NonEmptyString | undefined;
  email(): string;
  ms_user_id(): string;
  orcid(): NonEmptyString | undefined;
  institution_id(): string;
  static new(): PersonUpdateBuilder;
}
/**
 * Builder for [`PersonUpdate`](struct.PersonUpdate.html).
 */
export class PersonUpdateBuilder {
  private constructor();
  free(): void;
  grant_roles(value: any[]): PersonUpdateBuilder;
  revoke_roles(value: any[]): PersonUpdateBuilder;
  core(value: PersonUpdateCore): PersonUpdateBuilder;
  /**
   * Builds a new `PersonUpdate`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonUpdate;
}
export class PersonUpdateCore {
  private constructor();
  free(): void;
  id(): string;
  name(): NonEmptyString | undefined;
  email(): string;
  ms_user_id(): string;
  orcid(): NonEmptyString | undefined;
  institution_id(): string;
  static new(): PersonUpdateCoreBuilder;
}
/**
 * Builder for [`PersonUpdateCore`](struct.PersonUpdateCore.html).
 */
export class PersonUpdateCoreBuilder {
  private constructor();
  free(): void;
  id(value: string): PersonUpdateCoreBuilder;
  name(value?: NonEmptyString | null): PersonUpdateCoreBuilder;
  email(value?: string | null): PersonUpdateCoreBuilder;
  ms_user_id(value?: string | null): PersonUpdateCoreBuilder;
  orcid(value?: NonEmptyString | null): PersonUpdateCoreBuilder;
  institution_id(value?: string | null): PersonUpdateCoreBuilder;
  /**
   * Builds a new `PersonUpdateCore`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonUpdateCore;
}
export class PersonUpdateCoreError {
  private constructor();
  free(): void;
  error(): string;
}
export class PersonUpdateError {
  private constructor();
  free(): void;
  error(): string;
}
